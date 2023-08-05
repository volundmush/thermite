use std::{
    collections::{HashMap},
    time::{Duration, Instant}
};
use std::error::Error;

use tokio::{
    sync::mpsc::{Sender, Receiver, channel},
    task::JoinHandle,
    time
};

use tokio_stream::wrappers::IntervalStream;

use serde_json::Value as JsonValue;

use futures::{
    sink::{SinkExt},
    stream::{StreamExt}
};
use tokio::task::yield_now;

use crate::{
    protocols::link::protocol::{LinkStub}
};
use crate::msg::{Msg2Link, Msg2MudProtocol, Msg2Portal, Msg2PortalFromClient, Msg2PortalFromLink};
use crate::protocols::{ProtocolLink, MudData};


pub struct Portal {
    pub tx_portal: Sender<Msg2Portal>,
    rx_portal: Receiver<Msg2Portal>,
    link: Option<LinkStub>,
    clients: HashMap<usize, ProtocolLink>
}

impl Portal {
    pub fn new() -> Self {
        let (tx_portal, rx_portal) = channel(10);
        Self {
            tx_portal,
            rx_portal,
            clients: Default::default(),
            link: Default::default()
        }
    }

    pub async fn run(&mut self) {
        let mut interval_timer = IntervalStream::new(time::interval(Duration::from_millis(1000 * 20)));

        loop {
            tokio::select! {
                _ = interval_timer.next() => {
                    self.handle_interval_timer().await;
                }
                p_msg = self.rx_portal.recv() => {
                    match p_msg {
                        Some(msg) => {
                            let _ = self.handle_portal_message(msg).await;
                        }
                        None => {
                            println!("Portal: rx_portal.recv() returned None");
                        }
                    }
                }
            }

        }
    }

    async fn message_all_clients(&mut self, msg: &str) {
        for (conn_id, client) in self.clients.iter_mut() {
            let m = MudData {
                cmd: "text".to_string(),
                args: vec![JsonValue::String(msg.to_string())],
                kwargs: Default::default()
            };
            let _ = client.tx_protocol.send(Msg2MudProtocol::Data(vec![m])).await;
        }
    }

    async fn handle_interval_timer(&mut self) {
        if self.link.is_none() {
            let _ = self.message_all_clients("Portal awaiting connection from game server...\r\n").await;
        }
    }

    async fn handle_portal_message(&mut self, msg: Msg2Portal) {
        match msg {
            Msg2Portal::FromClient(conn_id, m) => {

                if let Some(client) = self.clients.get_mut(&conn_id) {
                    match m {
                        Msg2PortalFromClient::Capabilities(capa) => {
                            client.capabilities = capa.clone();
                            if let Some(link) = self.link.as_mut() {
                                let _ = link.tx_link.send(Msg2Link::ClientCapabilities(conn_id, capa)).await;
                            }
                        }
                        Msg2PortalFromClient::Data(data) => {
                            if let Some(link) = self.link.as_mut() {
                                let _ = link.tx_link.send(Msg2Link::ClientData(conn_id, data)).await;
                            }
                        }
                    }
                }
            }
            Msg2Portal::FromLink(conn_id, m) => {
                match m {
                    Msg2PortalFromLink::ClientMessage(client_id, m2) => {
                        if let Some(client) = self.clients.get_mut(&client_id) {
                            let _ = client.tx_protocol.send(Msg2MudProtocol::Data(m2)).await;
                        }
                    }
                    Msg2PortalFromLink::ClientDisconnected(client_id, reason) => {
                        if let Some(client) = self.clients.remove(&client_id) {
                            let _ = client.tx_protocol.send(Msg2MudProtocol::Disconnect).await;
                        }
                    }
                }
            },
            Msg2Portal::ClientDisconnected(conn_id, reason) => {
                if let Some(link) = self.link.as_mut() {
                    let _ = link.tx_link.send(Msg2Link::ClientDisconnected(conn_id, reason)).await;
                }
            },
            Msg2Portal::ClientConnected(stub) => {
                self.clients.insert(stub.conn_id, stub.clone());
                if let Some(link) = self.link.as_mut() {
                    let _ = link.tx_link.send(Msg2Link::ClientReady(stub)).await;
                } else {
                    let m = MudData {
                        cmd: "text".to_string(),
                        args: vec![JsonValue::String("Portal awaiting connection from game server...\r\n".to_string())],
                        kwargs: Default::default()
                    };
                    let _ = stub.tx_protocol.send(Msg2MudProtocol::Data(vec![m])).await;
                }
            },
            Msg2Portal::LinkConnected(stub) => {
                if let Some(link) = self.link.as_mut() {
                    let _ = link.tx_link.send(Msg2Link::Replaced).await;
                } else {
                    let _ = self.message_all_clients("Connection established to game server!\r\n").await;
                }
                self.link = Some(stub.clone());
                let _ = stub.tx_link.send(Msg2Link::ClientList(self.clients.clone())).await;
            },
            Msg2Portal::LinkDisconnected(conn_id, reason) => {
                self.link = None;
                let _ = self.message_all_clients("Connection to game server lost!\r\n").await;
            },
            Msg2Portal::Kill => {

            },
            Msg2Portal::Broadcast(msg) => {
                let _ = self.message_all_clients(&msg).await;
            }
        }
    }
}