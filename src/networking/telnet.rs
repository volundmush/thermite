use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    path::{PathBuf},
    sync::{Arc},
    sync::atomic::{AtomicUsize, Ordering},
    fs::File,
    io::BufReader
};
use std::io::Read;


use std::time::Duration;

use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender,
    io::{BufStream, AsyncBufRead, AsyncBufReadExt}
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
use crate::networking::CONNECTION_ID_COUNTER;

use crate::util::{ClientHelloStatus, check_tls_client_hello, check_http_request, HttpRequestStatus, generate_id};

pub struct TelnetAcceptor {
    listener: TcpListener,
    tls_acceptor: Option<Arc<TlsAcceptor>>,
    tx_portal: Sender<Msg2Portal>
}

impl TelnetAcceptor {
    pub async fn new(addr: SocketAddr, tls_acceptor: Option<Arc<TlsAcceptor>>, tx_portal: Sender<Msg2Portal>) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;

        Ok(TelnetAcceptor {
            listener,
            tls_acceptor,
            tx_portal
        })
    }

    pub async fn run(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    let mut handler = TelnetHandler::new(addr, self.tls_acceptor.clone(), self.tx_portal.clone());
                    tokio::spawn(async move {
                        handler.run(stream).await;
                    });
                }
                Err(e) => {
                    println!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

pub struct TelnetHandler {
    addr: SocketAddr,
    tls_option: Option<Arc<TlsAcceptor>>,
    tx_portal: Sender<Msg2Portal>
}

impl TelnetHandler {

    pub fn new(addr: SocketAddr, tls_option: Option<Arc<TlsAcceptor>>, tx_portal: Sender<Msg2Portal>) -> Self {
        Self {
            addr,
            tls_option,
            tx_portal
        }
    }

    pub async fn run(&mut self, stream: TcpStream) -> Result<(), Box<dyn Error>> {
        if let Some(tls_acceptor) = &self.tls_option {
            let hello_status = match timeout(Duration::from_millis(50), async {
                let mut buffer = vec![0; 5];

                loop {
                    stream.peek(&mut buffer).await.unwrap();

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

                    self.handle_telnet_connection(tls_stream, true).await?;
                }
                _ => {
                    // Invalid or timeout reached
                    // Handle non-TLS or invalid connections
                    self.handle_telnet_connection(stream, false).await?;
                }
            }
        } else {
            // No ServerConfig provided, handle the connection as a non-TLS connection
            self.handle_telnet_connection(stream, false).await?;
        }

        Ok(())
    }

    pub async fn handle_telnet_connection<S>(&mut self, mut socket: S,tls_engaged: bool) -> Result<(), Box<dyn std::error::Error>>
        where
            S: AsyncRead + AsyncReadExt + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let conn_id = CONNECTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let telnet_codec = Framed::new(socket, TelnetCodec::new(8192));

        let mut tel_prot = TelnetProtocol::new(conn_id, telnet_codec, self.addr.clone(), tls_engaged, self.tx_portal.clone());

        tel_prot.run().await;

        Ok(())
    }
}