use std::{
    net::SocketAddr
};

use tokio::{
    sync::mpsc::{Sender},
    net::{TcpListener, TcpStream},
};

use tokio_rustls::{
    TlsAcceptor,
    server::TlsStream
};


pub struct Listener {
    listen_id: String,
    listener: TcpListener,
    tls_acceptor: Option<TlsAcceptor>,
    tx_factory: Sender<Msg2Factory>
}


impl Listener {
    pub fn new(listen_id: String, listener: TcpListener, tls_acceptor: Option<TlsAcceptor>,
               tx_factory: Sender<Msg2Factory>) -> Self {
        Self {
            listen_id,
            tls_acceptor,
            listener,
            tx_factory,
        }
    }

    pub async fn run(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((tcp_stream, addr)) => {
                    if let Some(acceptor) = &self.tls_acceptor {
                        let acc = acceptor.clone();
                        if let Ok(tls_stream) = acc.accept(tcp_stream).await {
                            let _ = self.tx_factory.send(Msg2Factory::AcceptTLS(tls_stream, addr)).await;
                        }
                    } else {
                        let _ = self.tx_factory.send(Msg2Factory::AcceptTCP(tcp_stream, addr)).await;
                    }
                }
                Err(e) => {
                    eprintln!("Something went wrong with listener {}: {:?}", self.listen_id, e);
                }
            }
        }
    }
}

pub enum Msg2Factory {
    AcceptTCP(TcpStream, SocketAddr),
    AcceptTLS(TlsStream<TcpStream>, SocketAddr)
}