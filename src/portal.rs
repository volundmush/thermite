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

pub enum Msg2PortalFromClient {
    Capabilities(ProtocolCapabilities),
    Line(String),
    Lines(Vec<String>),
    GMCP(String, JsonValue),
}

pub enum Msg2PortalFromLink {
    ClientMessage(String, Msg2MudProtocol),
    ClientDisconnected(String),
    RequestPortalJson,
    SetPortalJson(JsonValue),
    RequestClientCapabilities(String),
    SetClientJson(String, JsonValue),
    RequestClientJson(String),
    RequestClientList
}

pub enum Msg2Portal {
    Kill,
    ClientConnected(ProtocolLink),
    ClientDisconnected(String, String),
    FromClient(String, Msg2PortalFromClient),
    LinkConnected(LinkStub),
    LinkDisconnected(String, String),
    FromLink(String, Msg2PortalFromLink),
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
        while let Some(f_msg) = self.rx_portal.recv().await {
            match f_msg {
                Msg2Portal::Kill => {
                    break;
                },
                Msg2Portal::LinkConnected(stub) => {
                    self.setup_stub(stub).await;
                },
                Msg2Portal::LinkDisconnected(id, reason) => {
                    self.clear_link().await;
                },
                Msg2Portal::ClientConnected(prot) => {
                    self.clients.insert(prot.conn_id.clone(), prot.clone());
                    if let Some(link) = &self.link {
                        let _ = link.tx_link.send(Msg2Link::ClientReady(prot)).await;
                    } else {
                        let _ = prot.tx_protocol.send(Msg2MudProtocol::Line(String::from("-- Connected to portal service. Waiting for game service...\n"))).await;
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
                Msg2Portal::FromClient(id, msg) => {
                    if let Some(p) = self.clients.get_mut(&id) {
                        if let Some(link) = &self.link {
                            match msg {
                                Msg2PortalFromClient::Capabilities(cap) => {
                                    println!("Portal Received Client Capabilities");
                                    p.capabilities = cap.clone();
                                    let _ = link.tx_link.send(Msg2Link::ClientCapabilities(id, cap)).await;
                                },
                                Msg2PortalFromClient::Line(s) => {
                                    let _ = link.tx_link.send(Msg2Link::ClientLine(id, s)).await;
                                },
                                Msg2PortalFromClient::Lines(lines) => {
                                    let _ = link.tx_link.send(Msg2Link::ClientLines(id, lines)).await;
                                },
                                Msg2PortalFromClient::GMCP(cmd, data) => {
                                    let _ = link.tx_link.send(Msg2Link::ClientGMCP(id, cmd, data)).await;
                                }
                            }
                        }
                    } else {
                        // I don't have that client. why don't I have that client?
                    }
                }
                Msg2Portal::PleaseWait => {
                    self.alert_clients_wait().await;
                },
                Msg2Portal::FromLink(id, msg) => {
                    if let Some(link) = &self.link {
                        match msg {
                            Msg2PortalFromLink::ClientMessage(cid, cmsg) => {
                                if let Some(p) = self.clients.get(&cid) {
                                    let _ = p.tx_protocol.send(cmsg).await;
                                }
                            },
                            Msg2PortalFromLink::ClientDisconnected(cid) => {
                                let mut clear = false;
                                if let Some(p) = self.clients.get(&cid) {
                                    clear = true;
                                    let _ = p.tx_protocol.send(Msg2MudProtocol::Disconnect).await;
                                }
                                if clear {
                                    self.clients.remove(&cid);
                                }
                            }
                            Msg2PortalFromLink::SetClientJson(cid, j) => {
                                if let Some(p) = self.clients.get_mut(&cid) {
                                    p.json_data = j;
                                }
                            },
                            Msg2PortalFromLink::RequestClientJson(cid) => {
                                if let Some(p) = self.clients.get(&cid) {
                                    let _ = link.tx_link.send(Msg2Link::ClientJson(id, p.json_data.clone())).await;
                                }
                            },
                            Msg2PortalFromLink::RequestClientCapabilities(cid) => {
                                if let Some(p) = self.clients.get(&cid) {
                                    let _ = link.tx_link.send(Msg2Link::ClientCapabilities(id, p.capabilities.clone())).await;
                                }
                            },
                            Msg2PortalFromLink::RequestClientList => {
                                let _ = link.tx_link.send(Msg2Link::ClientList(self.clients.clone())).await;
                            },
                            Msg2PortalFromLink::RequestPortalJson => {
                                let _ = link.tx_link.send(Msg2Link::PortalJson(self.json_data.clone())).await;
                            },
                            Msg2PortalFromLink::SetPortalJson(j) => {
                                self.json_data = j;
                            }
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
            for (id, stub) in &mut self.clients {
                let _ = stub.tx_protocol.send(Msg2MudProtocol::Line(String::from("-- Server connected!\n"))).await;
            }
        }
        self.link = Option::Some(stub);
        if let Some(s) = &self.link {
            let _ = s.tx_link.send(Msg2Link::ClientList(self.clients.clone())).await;
        }
    }

    async fn alert_clients_wait(&mut self) {
        for (id, stub) in &mut self.clients {
            let _ = stub.tx_protocol.send(Msg2MudProtocol::Line(String::from("-- Portal service is waiting on connection from game server...\n"))).await;
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