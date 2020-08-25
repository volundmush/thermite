use std::{
    collections::HashMap,
    net::SocketAddr
};

use tokio::{
    sync::mpsc::{channel, Receiver, Sender}
};

pub struct ClientConn {
    pub client_id: String,
    pub addr: SocketAddr
}

pub struct ClientCapabilities {

}

pub struct ConnectionData {
    pub addr: SocketAddr
}

pub enum Msg2GameServer {
    DisconnectClient(String, Option<String>)
}

pub struct GameServer {
    pub server_protocol: String,
    pub server_name: String,
    pub tx_server: Sender<Msg2GameServer>
}

pub struct ReqNewConn {
    pub server_type: String,
    pub server_name: String,
    pub conn_id: String,
    pub capabilities: ClientCapabilities,
    pub conn_data: ConnectionData
}

pub enum Msg2ConnManager {
    Kill,
    NewServer(GameServer),
    NewConn(ReqNewConn),
    Pong(String, String)
}

pub enum MsgFromConnManager {
    Disconnect(String, Option<String>),
    Ping(String)
}

pub struct ConnectionManager {
    pub servers: HashMap<String, GameClientServer>,
    pub clients: HashMap<String, ClientConn>,
    pub tx_connmanager: Sender<Msg2ConnManager>,
    pub rx_connmanager: Receiver<Msg2ConnManager>,
}

impl ConnectionManager {
    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_connmanager.recv().await {
                match msg {
                    Msg2ConnManager::Kill => {
                        break;
                    },
                    Msg2ConnManager::NewServer(srv) => {
                        self.accept_new_server(srv).await;
                    }
                }
            }
        }
    }

    async fn accept_new_server(&mut self, srv: GameServer) {

    }
}