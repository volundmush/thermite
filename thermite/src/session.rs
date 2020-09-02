use tokio::{
    prelude::*,
    sync::mpsc::{Receiver, Sender, channel},
};

use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

pub enum Msg2MudSession {
    Disconnect,
    Line(String),
    Prompt(String),
    Data,
    MSSP,
    Ready
}

pub struct SessionLink {
    pub conn_id: String,
    pub addr: SocketAddr,
    pub tls: bool,
    pub tx_session: Sender<Msg2MudSession>
}

pub enum Msg2SessionManager {
    NewSession(SessionLink),
    SessionCommand(String, String),
    SessionDisconnected(String),
}

pub struct SessionManager {
    sessions: HashMap<String, SessionLink>,
    pub tx_manager: Sender<Msg2SessionManager>,
    rx_manager: Receiver<Msg2SessionManager>
}

impl SessionManager {
    pub fn new() -> Self {
        let (tx_manager, rx_manager) = channel(50);

        Self {
            sessions: Default::default(),
            tx_manager,
            rx_manager
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_manager.recv().await {
                match msg {
                    Msg2SessionManager::NewSession(mut link) => {
                        let welcome = self.welcome_screen(&link);
                        link.tx_session.send(Msg2MudSession::Line(welcome));
                        self.sessions.insert(link.conn_id.clone(), link);
                    },
                    Msg2SessionManager::SessionCommand(conn_id, command) => {
                        println!("GOT COMMAND FROM {}: {}", conn_id, command);
                    },
                    Msg2SessionManager::SessionDisconnected(conn_id) => {
                        println!("SESSION {} DISCONNECTED!", conn_id);
                        let _ = self.sessions.remove(&conn_id);
                    }
                }
            }
        }
    }

    fn welcome_screen(&self, link: &SessionLink) -> String {
        String::from("NOT MUCH TO SEE YET!")
    }
}