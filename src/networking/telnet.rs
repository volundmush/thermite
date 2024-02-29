use std::{
    error::Error,
    net::SocketAddr,
    sync::Arc,
    sync::atomic::Ordering,

};

use std::time::Duration;

use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender,
};

use trust_dns_resolver::TokioAsyncResolver;

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};
use tokio::time::timeout;
use tokio_util::codec::Framed;
use crate::msg::Msg2Portal;
use crate::protocols::telnet::codec::TelnetCodec;
use crate::protocols::telnet::protocol::TelnetProtocol;
use crate::networking::CONNECTION_ID_COUNTER;

use crate::util::{ClientHelloStatus, check_tls_client_hello};

pub struct TelnetAcceptor {
    listener: TcpListener,
    tx_portal: Sender<Msg2Portal>
}

impl TelnetAcceptor {
    pub async fn new(addr: SocketAddr, tx_portal: Sender<Msg2Portal>) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;

        Ok(TelnetAcceptor {
            listener,
            tx_portal
        })
    }

    pub async fn run(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    let mut handler = TelnetHandler::new(addr, self.tx_portal.clone());
                    tokio::spawn(async move {
                        match handler.run(stream).await {
                            Ok(()) => {},
                            Err(e) => {
                                println!("Error accepting Telnet Connection: {}", e);
                            }
                        }
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
    tx_portal: Sender<Msg2Portal>,
    hostnames: Vec<String>
}

impl TelnetHandler {

    pub fn new(addr: SocketAddr, tx_portal: Sender<Msg2Portal>) -> Self {
        Self {
            addr,
            tx_portal,
            hostnames: Vec::new()
        }
    }

    pub async fn run(&mut self, stream: TcpStream) -> Result<(), Box<dyn Error>> {
        let resolver = TokioAsyncResolver::tokio_from_system_conf()?;

        if let Ok(response) = resolver.reverse_lookup(self.addr.ip()).await {
            self.hostnames = response.iter().map(|x| x.to_string()).collect();
        }

        self.handle_telnet_connection(stream, false).await?;

        Ok(())
    }

    pub async fn handle_telnet_connection<S>(&mut self, mut socket: S,tls_engaged: bool) -> Result<(), Box<dyn std::error::Error>>
        where
            S: AsyncRead + AsyncReadExt + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let conn_id = CONNECTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let telnet_codec = Framed::new(socket, TelnetCodec::new(8192));

        let mut tel_prot = TelnetProtocol::new(conn_id, telnet_codec, self.addr.clone(), self.hostnames.clone(), tls_engaged, self.tx_portal.clone());

        tel_prot.run().await;

        Ok(())
    }
}