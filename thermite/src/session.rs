use std::{
    net::SocketAddr,
    collections::HashMap,
    error::Error
};

use tokio::{
    prelude::*,
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle
};

use thermite_lib::conn::{
    Msg2SessionManager,
    Msg2Portal,
    Msg2Protocol,
    Msg2Session,
    ClientInfo,
    ClientCapabilities
};

pub struct Session {
    conn_id: String,
    tx_session: Sender<Msg2Session>,
    rx_session: Receiver<Msg2Session>,
    info: ClientInfo,
    capabilities: ClientCapabilities,
    tx_sessmanager: Sender<Msg2SessionManager>
}

impl Session {
    pub fn new(conn_id: String, info: ClientInfo, capabilities: ClientCapabilities,
               tx_session: Sender<Msg2Session>, rx_session: Receiver<Msg2Session>,
               tx_sessmanager: Sender<Msg2SessionManager>) -> Self {
        Self {
            conn_id,
            info,
            capabilities,
            tx_session,
            rx_session,
            tx_sessmanager
        }
    }

    pub async fn run(&mut self) {
        self.info.tx_protocol.send(Msg2Protocol::SessionReady(self.tx_session.clone())).await;

        loop {
            if let Some(msg) = self.rx_session.recv().await {
                match msg {
                    Msg2Session::Kill => {
                        break;
                    }
                    Msg2Session::ClientCapabilities(capabilities) => {
                        self.update_capabilities(capabilities).await;
                    },
                    Msg2Session::ClientDisconnected(reason) => {
                        let reason_copy = reason.clone();
                        if let Some(reason_2) = reason_copy {
                            println!("Session {} disconnected for reason: {}", self.conn_id, reason_2);
                        }
                        self.tx_sessmanager.send(Msg2SessionManager::ClientDisconnected(Some(self.conn_id.clone()))).await;
                        break;
                    },
                    Msg2Session::ClientCommand(command) => {
                        println!("Session {} received command from protocol: {}", self.conn_id, command);
                    }
                }
            }
        }
    }

    async fn update_capabilities(&mut self, capabilities: ClientCapabilities) {

    }
}

pub struct SessionLink {
    conn_id: String,
    tx_session: Sender<Msg2Session>,
    info: ClientInfo,
    handle: JoinHandle<()>
}

impl SessionLink {
    pub fn new(conn_id: String, info: ClientInfo, capabilities: ClientCapabilities,
               tx_sessmanager: Sender<Msg2SessionManager>) -> Result<Self, Box<dyn Error>> {
        let (tx_session, rx_session) = channel(50);

        // RUN SOME CODE HERE... check that IP address for bans for instance.
        // Return Err() if this connection needs to be kicked.

        let mut session = Session::new(conn_id.clone(), info.clone(),
                                       capabilities.clone(), tx_session.clone(),
                                       rx_session, tx_sessmanager);

        let handle = tokio::spawn(async move {session.run().await});

        Ok(Self {
            conn_id,
            tx_session,
            info: info,
            handle
        })
    }

    pub async fn run(&mut self) {

    }
}

pub struct SessionManager {
    sessions: HashMap<String, SessionLink>,
    rx_sessmanager: Receiver<Msg2SessionManager>,
    tx_sessmanager: Sender<Msg2SessionManager>,
    tx_portal: Sender<Msg2Portal>
}

impl SessionManager {
    pub fn new(tx_sessmanager: Sender<Msg2SessionManager>, rx_sessmanager: Receiver<Msg2SessionManager>,
               tx_portal: Sender<Msg2Portal>) -> Self {

        Self {
            sessions: Default::default(),
            tx_sessmanager,
            rx_sessmanager,
            tx_portal
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_sessmanager.recv().await {
                match msg {
                    Msg2SessionManager::Kill => {
                        for (k, v) in self.sessions.iter_mut() {
                            v.tx_session.send(Msg2Session::Kill).await;
                            break;
                        }
                    },
                    Msg2SessionManager::ClientReady(conn_id, info, capabilities) => {
                        println!("GOT A CLIENT! {}", conn_id);
                        if let Ok(link) = SessionLink::new(conn_id.clone(), info, capabilities, self.tx_sessmanager.clone()) {
                            self.sessions.insert(conn_id, link);
                        } else {
                            // This would only happen if SessionLink rejects the connection for some
                            // reason. Such as a ban.
                        }
                    },
                    Msg2SessionManager::ClientDisconnected(reason) => {

                    }
                }
            }
        }
    }
}