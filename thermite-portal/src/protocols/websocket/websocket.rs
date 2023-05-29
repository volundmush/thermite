use thermite_net::{Msg2Factory, FactoryLink};
use thermite_util::text::generate_id;
use serde_json::Value as JsonValue;
use tokio::{
    prelude::*,
    sync::mpsc::{Receiver, Sender, channel},
    time
};
use tokio_tungstenite::{accept_async};

use crate::{Msg2ProtocolManager, Msg2MudProtocol, ProtocolLink,
    ProtocolCapabilities};

pub struct WebSocketProtocol {
    running: bool
}

impl WebSocketProtocol {

    pub fn new() -> Self {
        
    }

    pub async fn run(&mut self) {

    }
}


pub struct WebSocketProtocolFactory {
    factory_id: String,
    pub tx_factory: Sender<Msg2Factory>,
    rx_factory: Receiver<Msg2Factory>,
    tx_manager: Sender<Msg2ProtocolManager>,
    ids: HashSet<String>,
}

impl WebSocketProtocolFactory {
    pub fn new(factory_id: String, tx_manager: Sender<Msg2ProtocolManager>) -> Self {
        let (tx_factory, rx_factory) = channel(50);

        Self {
            factory_id,
            tx_factory,
            rx_factory,
            tx_manager,
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
        where C: AsyncRead + AsyncWrite + Send + 'static + Unpin + std::marker::Sync
    {
        let gen_id = generate_id(12, &self.ids);
        let conn_id = format!("{}_{}", self.factory_id, gen_id);
        self.ids.insert(gen_id);
    }
}