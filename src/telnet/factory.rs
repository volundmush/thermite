use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::Arc
};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc::{Receiver, Sender, channel}
};

use tokio_util::codec::{Framed};

use bytes::{BytesMut, Bytes};


use crate::{
    telnet::{
        protocol::{TelnetOption, TelnetOptionState, TelnetProtocol, TelnetHandshakes},
        codec::TelnetCodec,
        codes as tc
    },
    net::{FactoryLink, Msg2Factory},
    util::generate_id,
    portal::{Msg2Portal}
};

pub struct TelnetProtocolFactory {
    factory_id: String,
    pub tx_factory: Sender<Msg2Factory>,
    rx_factory: Receiver<Msg2Factory>,
    tx_portal: Sender<Msg2Portal>,
    ids: HashSet<String>
}

impl TelnetProtocolFactory {
    pub fn new(factory_id: &str, tx_portal: Sender<Msg2Portal>) -> Self {
        let (tx_factory, rx_factory) = channel(50);

        Self {
            factory_id: String::from(factory_id),
            tx_factory,
            rx_factory,
            tx_portal,
            ids: HashSet::default(),
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
                        self.accept(stream, addr, true);
                    },
                    Msg2Factory::AcceptTCP(stream, addr) => {
                        self.accept(stream, addr, false);
                    },
                    Msg2Factory::Kill => {
                        break;
                    }
                }
            }
        }
    }

    fn accept<C>(&mut self, conn: C, addr: SocketAddr, tls: bool)
        where C: AsyncRead + AsyncWrite + Send + 'static + Unpin + Sync
    {
        let telnet_codec = Framed::new(conn, TelnetCodec::new(8192));
        let gen_id = generate_id(12, &self.ids);
        let conn_id = format!("{}_{}", self.factory_id, gen_id);
        self.ids.insert(gen_id);

        let mut tel_prot = TelnetProtocol::new(conn_id, telnet_codec, addr.clone(), tls.clone(), self.tx_portal.clone());

        let _ = tokio::spawn(async move {tel_prot.run().await;});
    }
}