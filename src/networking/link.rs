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
use crate::msg::{Msg2Link, Msg2Portal};
use crate::protocols::link::protocol::{LinkProtocol, LinkStub};
use crate::networking::CONNECTION_ID_COUNTER;

use crate::util::{ClientHelloStatus, check_tls_client_hello, check_http_request, HttpRequestStatus, generate_id};

use tokio_tungstenite::{tungstenite, WebSocketStream, accept_async};
use tungstenite::Error as WsError;

pub struct LinkAcceptor {
    listener: TcpListener,
    tx_portal: Sender<Msg2Portal>
}

impl LinkAcceptor {
    pub async fn new(addr: SocketAddr, tx_portal: Sender<Msg2Portal>) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;

        Ok(LinkAcceptor {
            listener,
            tx_portal
        })
    }

    pub async fn run(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    let mut handler = LinkHandler::new(addr, self.tx_portal.clone());
                    tokio::spawn(async move {
                        match handler.run(stream).await {
                            Ok(()) => {},
                            Err(e) => {
                                println!("Error accepting link connection: {}", e);
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

pub struct LinkHandler {
    addr: SocketAddr,
    tx_portal: Sender<Msg2Portal>
}

impl LinkHandler {

    pub fn new(addr: SocketAddr, tx_portal: Sender<Msg2Portal>) -> Self {
        Self {
            addr,
            tx_portal
        }
    }

    pub async fn run(&mut self, stream: TcpStream) -> Result<(), Box<dyn Error>> {

        let ws_stream = accept_async(stream).await?;
        let conn_id = CONNECTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        let (tx_link, rx_link) = tokio::sync::mpsc::channel::<Msg2Link>(100);

        let link_stub = LinkStub {
            conn_id,
            addr: self.addr.clone(),
            tls: false,
            tx_link
        };

        let _ = self.tx_portal.send(Msg2Portal::LinkConnected(link_stub)).await;

        let mut link_protocol = LinkProtocol::new(conn_id, ws_stream, self.addr.clone(), false, self.tx_portal.clone(), rx_link);
        let _ = link_protocol.run().await;

        Ok(())
    }


}