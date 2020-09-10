use tokio::{
    prelude::*,
    sync::mpsc::{Receiver, Sender, channel},
    sync::oneshot,
    net::TcpStream
};

use std::{
    collections::{HashMap},
    net::SocketAddr,
    error::Error
};

use thermite_protocol::{
    ProtocolCapabilities,
    ProtocolLink,
    Msg2MudProtocol,
    ConnectResponse,
    Msg2Game
};

pub struct ClientData {
    pub client_id: String,
    pub address: SocketAddr,
    pub capabilities: ProtocolCapabilities,
    pub json_data: serde_json::Value
}

impl From<ProtocolLink> for ClientData {
    fn from(src: ProtocolLink) -> Self {
        Self {
            client_id: src.conn_id,
            address: src.addr,
            capabilities: src.capabilities,
            json_data: src.json_data
        }
    }
}

pub enum Msg2GameLink {
    Kill,
    Line(String, String),
    Prompt(String, String),
    GMCP(String, String, serde_json::Value),
    RequestServerStatus,
    NewClient(ClientData),
    Disconnect(String),
    UpdateJson(String, serde_json::Value),
    UpdateAllJson(HashMap<String, serde_json::Value>),
    UpdateCapabilities(String, ProtocolCapabilities),
    UpdateAll(Vec<ClientData>)
}

#[derive(Debug)]
pub enum PortalMode {
    Inactive,
    Active,
    Copyover
}

pub enum Msg2Portal {
    Kill,
    NewLink(TcpStream),
    Line(String, String),
    Prompt(String, String),
    GMCP(String, String, serde_json::Value),
    ServerStatus(String, HashMap<String, String>),
    Disconnect(String),
    UpdateJson(String, serde_json::Value),
    UpdateAllJson(HashMap<String, serde_json::Value>),
    RequestCapabilities(String),
    RequestJson(String),
    RequestAll,

}

#[derive(Debug)]
pub struct Portal {
    pub game_id: String,
    pub game_name: String,
    protocols: HashMap<String, ProtocolLink>,
    pub tx_game: Sender<Msg2Game>,
    rx_game: Receiver<Msg2Game>,
    rx_portal: Receiver<Msg2Portal>,
    pub tx_portal: Sender<Msg2Portal>,
    tx_link: Sender<Msg2GameLink>,
    running: bool,
    mode: PortalMode
}

impl Portal {
    pub fn new(game_id: String, game_name: String, ) -> Self {

        let (tx_game, rx_game) = channel(50);
        let (tx_portal, rx_portal) = channel(50);
        let (tx_link, rx_link) = channel(50);

        Self {
            game_id,
            game_name,
            protocols: Default::default(),
            tx_game,
            rx_game,
            tx_portal,
            rx_portal,
            tx_link,
            running: true,
            mode: PortalMode::Inactive
        }
    }

    pub async fn run(&mut self) {
        while self.running {
            tokio::select! {
                g_msg = self.rx_game.recv() => {
                    if let Some(msg) = g_msg {
                        let _ = self.process_game_message(msg).await;
                    }
                }
            }
        }
    }

    async fn process_game_message(&mut self, msg: Msg2Game) {
        match msg {

        }
    }

    async fn new_protocol(&mut self, link: ProtocolLink, send: oneshot::Sender<ConnectResponse>) {
        // There will be some logic here for checking if this IP address should be allowed to connect...
        // For now, just allow it.
        let mut tx = link.tx_protocol.clone();
        self.protocols.insert(link.conn_id.clone(), link);
        let _ = send.send(ConnectResponse::Ok);
        let _ = tx.send(Msg2MudProtocol::Line(welcome)).await;
    }
}