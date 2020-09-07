use tokio::{
    prelude::*,
    sync::mpsc::{Receiver, Sender, channel},
    sync::oneshot,
};

use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    error::Error
};

use crate::{Msg2ProtocolManager, ProtocolCapabilities, ProtocolLink, Msg2Game, 
    Msg2MudProtocol, ConnectResponse, MudProtocolFilter};


#[derive(Debug)]
pub struct ProtocolManager {
    pub tx_prot_manager: Sender<Msg2ProtocolManager>,
    rx_prot_manager: Receiver<Msg2ProtocolManager>,
    tx_game: Sender<Msg2Game>,
    protocols: HashMap<String, ProtocolLink>,
    running: bool,
}

impl ProtocolManager {
    pub fn new(tx_game: Sender<Msg2Lobby>) -> Self {
        let (tx_prot_manager, rx_prot_manager) = channel(50);

        Self {
            tx_prot_manager,
            rx_prot_manager,
            tx_game,
            running: true,
            protocols: Default::default()
        }
    }

    pub async fn run(&mut self) {
        while self.running {
            tokio::select! {
                prot_msg = self.rx_prot_manager.recv() => {
                    if let Some(msg) = prot_msg {
                        let _ = self.process_protocol_message(msg).await;
                    }
                },
                }
            }
        }
    }

    async fn process_connection_message(&mut self, msg: Msg2ConnManager) {

    }

    async fn process_protocol_message(&mut self, msg: Msg2ProtocolManager) {
        println!("Protocol Manager got a message: {:?}", msg);
        match msg {
            Msg2ProtocolManager::NewProtocol(mut link, sender) => {
                let _ = self.accept_protocol(link, sender).await;
            },
            Msg2ProtocolManager::ProtocolCommand(conn_id, command) => {
                let _ = self.execute_command(conn_id, command).await;
            },
            Msg2ProtocolManager::ProtocolDisconnected(conn_id) => {
                println!("SESSION {} DISCONNECTED!", conn_id);
                self.protocols.remove(&conn_id);
                let _ = self.tx_game.send(Msg2Game:ProtocolDisconnected(conn_id)).await;
            }
        }
    }

    async fn accept_protocol(&mut self, link: ProtocolLink, sender: oneshot::Sender<ConnectResponse>) {
        // Intercept the sender oneshot and create a new one. We'll be standing in for the Lobby here.
        let (mut tx_response, mut rx_response) = oneshot::channel::<ConnectResponse>();
        let _ = self.tx_lobby.send(Msg2Lobby::NewProtocol(link.clone(), tx_response)).await;

        match rx_response.await {
            Ok(answer) => {
                match &answer {
                    ConnectResponse::Ok => {
                        // The lobby has accepted this connection. We'll add it to our tracking.
                        self.protocols.insert(link.conn_id.clone(), link);
                    },
                    ConnectResponse::Error(reason) => {
                        // the lobby has rejected this connection. This protocol will not be accepted.
                        // The answer is passed on to the protocol, which will disconnect.
                    }
                }
                let _ = sender.send(answer).await;
            },
            Err(e) => {

            }
        }
    }

    async fn execute_command(&mut self, conn_id: String, command: String) {
        println!("GOT COMMAND FROM {}: {}", conn_id, command);

        // letting the above stand for debugging/reference right now.
        if self.state.protocols.contains_key(&conn_id) {
            let prot = self.state.protocols.get().unwrap().clone();
            if command.starts_with("-") || command.starts_with(".") {
                // Commands that begin with - or . are Thermite commands. Route them to the lobby.
                let _ = self.tx_game.send(Msg2Game::ProtocolCommand(conn_id, command)).await;
            } else {
                // Other commands are routed to a game, if it's connected...
            }
        } else {
            // This is what happens if we somehow get a command but there's no protocol registered...
            // I'm not sure what that should be, TBH.
        }
    }
}