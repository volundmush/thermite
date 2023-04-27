use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    path::{PathBuf},
    sync::{Arc},
    sync::atomic::{AtomicUsize, Ordering},
    fs::File,
    io::BufReader
};


use std::time::Duration;

use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender
};

use tokio_rustls::rustls;
use tokio_rustls::{TlsStream, TlsAcceptor};

use tokio_rustls::rustls::{
    Certificate, PrivateKey, ServerConfig,
    server::NoClientAuth
};



use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};
use tokio::time::timeout;
use tokio_util::codec::Framed;
use crate::msg::Msg2Portal;
use crate::protocols::telnet::codec::TelnetCodec;
use crate::protocols::telnet::protocol::TelnetProtocol;

use crate::util::{ClientHelloStatus, check_tls_client_hello, check_http_request, HttpRequestStatus, generate_id};

pub struct ExternalAcceptor {
    listener: TcpListener,
    tls_acceptor: Option<Arc<TlsAcceptor>>,
    tx_portal: Sender<Msg2Portal>
}

impl ExternalAcceptor {
    pub async fn new(addr: SocketAddr, pem: Option<PathBuf>, key: Option<PathBuf>, tx_portal: Sender<Msg2Portal>) -> ExternalAcceptor {
        let listener = TcpListener::bind(addr).await.unwrap();

        let tls_acceptor = if pem.is_some() && key.is_some() {
            let cert_file = File::open(pem.unwrap().to_str().unwrap()).unwrap();
            let cert_reader = BufReader::new(cert_file);
            let certs = Certificate::from_pem(cert_reader).unwrap();

            // Read the private key file
            let key_file = File::open(key.unwrap().to_str().unwrap()).unwrap();
            let key_reader = BufReader::new(key_file);
            let key = PrivateKey::from_pem(key_reader).unwrap();

            let mut config = ServerConfig::new(NoClientAuth::new());
            config.set_single_cert(certs, key).unwrap();
            let mut tls_acceptor = TlsAcceptor::from(config);
            Some(Arc::new(tls_acceptor))
        } else {
            None
        };

        ExternalAcceptor {
            listener,
            tls_acceptor,
            tx_portal
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let (stream, addr) = self.listener.accept().await?;
            tokio::spawn(async move {
                run_external_handler(addr, self.tls_acceptor.clone(), stream, self.tx_portal.clone()).await;
            });
        }
    }
}


pub async fn run_external_handler(
    addr: SocketAddr,
    tls_option: Option<Arc<TlsAcceptor>>,
    stream: TcpStream,
    tx_portal: Sender<Msg2Portal>
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tls_acceptor) = tls_option.as_ref() {
        let hello_status = match timeout(Duration::from_millis(50), async {
            let mut buffer = vec![0; 5];

            loop {
                stream.peek(&mut buffer).await.expect("TODO: panic message");

                let status = check_tls_client_hello(&buffer);
                match status {
                    ClientHelloStatus::Complete | ClientHelloStatus::Invalid => {
                        break status;
                    }
                    ClientHelloStatus::Partial => {
                        // Yield the current task to prevent a busy loop
                        tokio::task::yield_now().await;
                    }
                }
            }
        }).await {
            Ok(status) => status,
            Err(_) => ClientHelloStatus::Invalid
        };

        match hello_status {
            ClientHelloStatus::Complete => {
                let tls_stream = tls_acceptor.as_ref().accept(stream).await?;

                handle_external_connection(addr, tls_stream, true, tx_portal).await?;
            }
            _ => {
                // Invalid or timeout reached
                // Handle non-TLS or invalid connections
                handle_external_connection(addr,  stream, false, tx_portal).await?;
            }
        }
    } else {
        // No ServerConfig provided, handle the connection as a non-TLS connection
        handle_external_connection(addr, stream, false, tx_portal).await?;
    }

    Ok(())
}

static CONNECTION_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub async fn handle_external_connection<S>(
    addr: SocketAddr,
    mut socket: S,
    tls_engaged: bool,
    tx_portal: Sender<Msg2Portal>
) -> Result<(), Box<dyn std::error::Error>>
    where
        S: AsyncRead + AsyncReadExt + AsyncWrite + Unpin + Send + Sync + 'static,
{
    // This function is called when a new connection is established. It might be TCP or TLS.

    let mut buffer = [0; 512];
    let mut bytes_peeked = 0;

    let conn_id = CONNECTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Use a timeout to check for an HTTP request
    let http_status = tokio::time::timeout(
        Duration::from_millis(50),
        async {
            loop {
                bytes_peeked = socket.peek(&mut buffer).await.unwrap_or(0);
                match check_http_request(&buffer[..bytes_peeked]) {
                    HttpRequestStatus::Complete | HttpRequestStatus::Invalid => break,
                    HttpRequestStatus::Partial => {
                        // Yield to the scheduler to allow other tasks to run
                        tokio::task::yield_now().await;
                    }
                }
            }
            check_http_request(&buffer[..bytes_peeked])
        },
    ).await.unwrap_or(HttpRequestStatus::Invalid);

    match http_status {
        HttpRequestStatus::Complete => {
            // HTTP request detected
            handle_http(addr, socket, tls_engaged, conn_id, tx_portal).await
        },
        _ => {
            // Either not an HTTP request or timed out, assuming "MUD Telnet"
            handle_telnet(addr, socket, tls_engaged, conn_id, tx_portal).await
        }
    }
}

pub async fn handle_http<S>(
    addr: SocketAddr,
    socket: S,
    tls_engaged: bool,
    conn_id: usize,
    tx_portal: Sender<Msg2Portal>
) -> Result<(), Box<dyn std::error::Error>>
    where
        S: AsyncRead + AsyncWrite + AsyncReadExt + Unpin + Send + Sync + 'static,
{
    // Same code as in the original ExternalHandler::handle_http()
    Ok(())
}

pub async fn handle_telnet<S>(
    addr: SocketAddr,
    socket: S,
    tls_engaged: bool,
    conn_id: usize,
    tx_portal: Sender<Msg2Portal>
) -> Result<(), Box<dyn std::error::Error>>
    where
        S: AsyncRead + AsyncWrite + AsyncReadExt + Unpin + Send + Sync + 'static,
{

    let telnet_codec = Framed::new(socket, TelnetCodec::new(8192));

    let mut tel_prot = TelnetProtocol::new(conn_id, telnet_codec, addr, tls_engaged, tx_portal);

    tel_prot.run().await;

    Ok(())
}
