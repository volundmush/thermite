use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    path::{PathBuf}
};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender
};
use crate::msg::Msg2Portal;

pub struct InternalAcceptor {
    listener: TcpListener,
    tx_portal: Sender<Msg2Portal>
}

impl InternalAcceptor {
    pub async fn new(addr: SocketAddr, tx_portal: Sender<Msg2Portal>) -> InternalAcceptor {
        let listener = TcpListener::bind(addr).await.unwrap();
        InternalAcceptor {
            listener,
            tx_portal
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let (stream, addr) = self.listener.accept().await?;

        }
    }
}

