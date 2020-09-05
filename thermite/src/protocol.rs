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

use crate::db::{
    Msg2DbManager
};

pub struct ProtocolManager {
    protocols: HashMap<String, ProtocolLink>,
    pub tx_manager: Sender<Msg2ProtocolManager>,
    rx_manager: Receiver<Msg2ProtocolManager>,
    tx_db: Sender<Msg2DbManager>
}

impl ProtocolManager {
    pub fn new(tx_db: Sender<Msg2DbManager>) -> Self {
        let (tx_manager, rx_manager) = channel(50);

        Self {
            protocols: Default::default(),
            tx_manager,
            rx_manager,
            tx_db
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_manager.recv().await {
                println!("Protocol Manager got a message: {:?}", msg);
                match msg {
                    Msg2ProtocolManager::NewProtocol(mut link) => {
                        let welcome = self.welcome_screen(&link);
                        let mut tx = link.tx_protocol.clone();
                        self.protocols.insert(link.conn_id.clone(), link);
                        let _ = tx.send(Msg2MudProtocol::Ready).await;
                        let _ = tx.send(Msg2MudProtocol::Line(welcome)).await;
                    },
                    Msg2ProtocolManager::ProtocolCommand(conn_id, command) => {
                        println!("GOT COMMAND FROM {}: {}", conn_id, command);
                        if let Some(link) = self.protocols.get_mut(&conn_id) {
                            let _ = link.tx_protocol.send(Msg2MudProtocol::Line(format!("ECHO: {}", command))).await;
                        }
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