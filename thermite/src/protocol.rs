use tokio::{
    prelude::*,
    sync::mpsc::{Receiver, Sender, channel},
};

use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

use thermite_protocol::{
    Msg2ProtocolManager,
    ProtocolCapabilities,
    ProtocolLink,
    Msg2MudProtocol,
};

pub struct ProtocolManager {
    protocols: HashMap<String, ProtocolLink>,
    pub tx_manager: Sender<Msg2ProtocolManager>,
    rx_manager: Receiver<Msg2ProtocolManager>
}

impl ProtocolManager {
    pub fn new() -> Self {
        let (tx_manager, rx_manager) = channel(50);

        Self {
            protocols: Default::default(),
            tx_manager,
            rx_manager
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_manager.recv().await {
                match msg {
                    Msg2ProtocolManager::NewProtocol(mut link) => {
                        let welcome = self.welcome_screen(&link);
                        let _ = link.tx_protocol.send(Msg2MudProtocol::Line(welcome));
                        self.protocols.insert(link.conn_id.clone(), link);
                    },
                    Msg2ProtocolManager::ProtocolCommand(conn_id, command) => {
                        println!("GOT COMMAND FROM {}: {}", conn_id, command);
                    },
                    Msg2ProtocolManager::ProtocolDisconnected(conn_id) => {
                        println!("SESSION {} DISCONNECTED!", conn_id);
                        self.protocols.remove(&conn_id);
                    }
                }
            }
        }
    }

    fn welcome_screen(&self, link: &ProtocolLink) -> String {
        String::from("NOT MUCH TO SEE YET!")
    }
}