use std::{
    collections::{HashSet},
    net::SocketAddr
};

use tokio::{
    sync::mpsc::{Sender, Receiver, channel},
    io::{AsyncRead, AsyncWrite}
};

use tokio_tungstenite::{accept_async, tungstenite::Error};

use crate::{
    net::{Msg2Factory, FactoryLink},
    portal::Msg2Portal,
    util::generate_id,
    link::{
        protocol::{LinkProtocol, LinkStub}
    }
};

pub struct LinkProtocolFactory {
    factory_id: String,
    pub tx_factory: Sender<Msg2Factory>,
    rx_factory: Receiver<Msg2Factory>,
    tx_portal: Sender<Msg2Portal>,
    ids: HashSet<String>,
}

impl LinkProtocolFactory {
    pub fn new(factory_id: &str, tx_portal: Sender<Msg2Portal>) -> Self {
        let (tx_factory, rx_factory) = channel(50);

        Self {
            factory_id: String::from(factory_id),
            tx_factory,
            rx_factory,
            tx_portal,
            ids: HashSet::default()
        }
    }

    pub fn link(&self) -> FactoryLink {
        FactoryLink {
            factory_id: self.factory_id.clone(),
            tx_factory: self.tx_factory.clone()
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(f_msg) = self.rx_factory.recv().await {
                match f_msg {
                    Msg2Factory::AcceptTLS(stream, addr) => {
                        self.accept(stream, addr, true).await;
                    },
                    Msg2Factory::AcceptTCP(stream, addr) => {
                        self.accept(stream, addr, false).await;
                    },
                    Msg2Factory::Kill => {
                        break;
                    }
                }
            }
        }
    }

    async fn accept<C>(&mut self, conn: C, addr: SocketAddr, tls: bool)
        where C: AsyncRead + AsyncWrite + Send + 'static + Unpin + Sync
    {
        match accept_async(conn).await {
            Ok(ws) => {
                let gen_id = generate_id(12, &self.ids);
                let conn_id = format!("{}_{}", self.factory_id, gen_id);
                self.ids.insert(gen_id);
                let (tx_link, rx_link) = channel(10);

                let mut prot = LinkProtocol::new(conn_id.clone(), ws,
                                                 addr.clone(), tls.clone(),
                                                 self.tx_portal.clone(), rx_link);
                let link = LinkStub {
                    addr: addr.clone(),
                    conn_id,
                    tls,
                    tx_link
                };
                let _ = tokio::spawn(async move {prot.run().await});
                let _ = self.tx_portal.send(Msg2Portal::LinkConnected(link)).await;
            },
            Err(e) => {

            }
        }



    }
}