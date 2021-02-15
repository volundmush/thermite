use std::{
    collections::{HashMap},
    time::Duration
};

use tokio::{
    sync::mpsc::{Sender, Receiver, channel},
    task::JoinHandle,
    time
};

use serde_json::Value as JsonValue;

use crate::{
    net::{Msg2MudProtocol, ProtocolCapabilities, ProtocolLink},
    link::protocol::{LinkStub, Msg2Link}
};

pub enum Msg2Portal {
    Kill,
    ClientReady(ProtocolLink),
    ClientDisconnected(String, String),
    ClientCapabilities(String, ProtocolCapabilities),
    ClientLine(String, String),
    ClientLines(String, Vec<String>),
    ClientGMCP(String, String, JsonValue),
    LinkReady(LinkStub),
    LinkDisconnected(String, String),
    LinkRequestClients,
    LinkClientJson(String, JsonValue),
    LinkClientRequestJson(String),
    LinkClientRequestCapabilities(String),
    LinkDumpJson(JsonValue),
    LinkRequestJson,
    LinkClientMessage(String, Msg2MudProtocol),
    PleaseWait
}

pub struct Portal {
    pub tx_portal: Sender<Msg2Portal>,
    rx_portal: Receiver<Msg2Portal>,
    link: Option<LinkStub>,
    clients: HashMap<String, ProtocolLink>,
    wait_task: Option<JoinHandle<()>>,
    json_data: JsonValue
}

impl Portal {
    pub fn new() -> Self {
        let (tx_portal, rx_portal) = channel(10);
        Self {
            tx_portal,
            rx_portal,
            clients: Default::default(),
            link: Default::default(),
            wait_task: Default::default(),
            json_data: Default::default()
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(f_msg) = self.rx_portal.recv().await {
                match f_msg {
                    Msg2Portal::Kill => {
                        break;
                    },
                    Msg2Portal::LinkReady(stub) => {
                        self.setup_stub(stub).await;
                    },
                    Msg2Portal::LinkDisconnected(id, reason) => {
                        self.clear_link().await;
                    },
                    Msg2Portal::ClientReady(prot) => {
                        self.clients.insert(prot.conn_id.clone(), prot.clone());
                        if let Some(link) = &self.link {
                            let _ = link.tx_link.send(Msg2Link::ClientReady(prot)).await;
                        } else {
                            let _ = prot.tx_protocol.send(Msg2MudProtocol::Line(String::from("Connected to portal service. Waiting for game service...\n"))).await;
                        }
                    },
                    Msg2Portal::LinkClientMessage(id, msg) => {
                        if let Some(p) = self.clients.get_mut(&id) {
                            let _ = p.tx_protocol.send(msg).await;
                        } else {
                            // I don't have that client. why don't I have that client?
                        }
                    },
                    Msg2Portal::ClientDisconnected(id, reason) => {
                        let mut clear = true;
                        if self.clients.contains_key(&id) {
                            clear = true;
                            if let Some(link) = &self.link {
                                let _ = link.tx_link.send(Msg2Link::ClientDisconnected(id.clone(), reason)).await;
                            }
                        } else {
                            // I don't have that client. why don't I have that client?
                        }
                        if clear {
                            let _ = self.clients.remove(&id);
                        }
                    },
                    Msg2Portal::ClientCapabilities(id, cap) => {
                        if let Some(p) = self.clients.get_mut(&id) {
                            if let Some(link) = &self.link {
                                p.capabilities = cap.clone();
                                let _ = link.tx_link.send(Msg2Link::ClientCapabilities(id, cap)).await;
                            }
                        } else {
                            // I don't have that client. why don't I have that client?
                        }
                    },
                    Msg2Portal::ClientLines(id, lines) => {
                        if let Some(p) = self.clients.get_mut(&id) {
                            if let Some(link) = &self.link {
                                let _ = link.tx_link.send(Msg2Link::ClientLines(id, lines)).await;
                            }
                        } else {
                            // I don't have that client. why don't I have that client?
                        }
                    },
                    Msg2Portal::ClientLine(id, line) => {
                        if let Some(p) = self.clients.get_mut(&id) {
                            if let Some(link) = &self.link {
                                let _ = link.tx_link.send(Msg2Link::ClientLine(id, line)).await;
                            }
                        } else {
                            // I don't have that client. why don't I have that client?
                        }
                    },
                    Msg2Portal::ClientGMCP(id, cmd, data) => {
                        if let Some(p) = self.clients.get_mut(&id) {
                            if let Some(link) = &self.link {
                                let _ = link.tx_link.send(Msg2Link::ClientGMCP(id, cmd, data)).await;
                            }
                        } else {
                            // I don't have that client. why don't I have that client?
                        }
                    },
                    Msg2Portal::PleaseWait => {
                        self.alert_clients_wait().await;
                    },
                    Msg2Portal::LinkRequestClients => {
                        if let Some(link) = &self.link {
                            let _ = link.tx_link.send(Msg2Link::ClientData(self.clients.clone())).await;
                        }
                    },
                    Msg2Portal::LinkClientJson(id, j) => {
                        if let Some(p) = self.clients.get_mut(&id) {
                            p.json_data = j;
                        } else {
                            // I don't have that client. why don't I have that client?
                        }
                    },
                    Msg2Portal::LinkClientRequestJson(id) => {
                        if let Some(p) = self.clients.get(&id) {
                            if let Some(link) = &self.link {
                                let _ = link.tx_link.send(Msg2Link::ClientJson(id, p.json_data.clone())).await;
                            }
                        } else {
                            // I don't have that client. why don't I have that client?
                        }
                    },
                    Msg2Portal::LinkRequestJson => {
                        if let Some(link) = &self.link {
                            let _ = link.tx_link.send(Msg2Link::PortalJson(self.json_data.clone())).await;
                        }
                    },
                    Msg2Portal::LinkDumpJson(j) => {
                        self.json_data = j;
                    },
                    Msg2Portal::LinkClientRequestCapabilities(id) => {
                        if let Some(p) = self.clients.get(&id) {
                            if let Some(link) = &self.link {
                                let _ = link.tx_link.send(Msg2Link::ClientCapabilities(id, p.capabilities.clone())).await;
                            }
                        } else {
                            // I don't have that client. why don't I have that client?
                        }
                    }
                }
            }
        }
    }

    async fn setup_stub(&mut self, stub: LinkStub) {
        if let Some(s) = &mut self.link {
            let _ = s.tx_link.send(Msg2Link::Replaced).await;
        }
        let mut cancel = false;
        if let Some(h) = &mut self.wait_task {
            h.abort();
            cancel = true;
        }
        if cancel {
            self.wait_task = None;
        }
        self.link = Option::Some(stub);
        if let Some(s) = &self.link {
            let _ = s.tx_link.send(Msg2Link::ClientData(self.clients.clone())).await;
        }
    }

    async fn alert_clients_wait(&mut self) {
        for (id, stub) in &mut self.clients {
            let _ = stub.tx_protocol.send(Msg2MudProtocol::Line(String::from("Portal service is waiting on connection from game server...\n"))).await;
        }
    }

    async fn clear_link(&mut self) {
        if self.link.is_some() {
            self.link = None;
            //self.alert_clients_wait().await;
        }
        self.start_timer().await;
    }

    pub async fn start_timer(&mut self) {
        let send_chan = self.tx_portal.clone();
        self.wait_task = Some(tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(5000));
            loop {
                interval.tick().await;
                let _ = send_chan.send(Msg2Portal::PleaseWait).await;
            }
        }));
    }

}