use std::{
    net::SocketAddr,
    collections::HashMap,
    str::FromStr
};

use tokio::{
    sync::mpsc::{Sender, Receiver},
    io::{AsyncRead, AsyncWrite}
};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use futures::{StreamExt, SinkExt};


use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Message as WsMessage;
use crate::msg::{Msg2Link, Msg2MudProtocol, Msg2Portal, Msg2PortalFromLink};
use crate::protocols::{ProtocolCapabilities, ProtocolData, MudData};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionDisconnect {
    pub kind: String,
    pub id: usize,
    pub reason: String
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgRequestCapabilities {
    pub kind: String,
    pub id: usize
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgJson {
    pub kind: String,
    pub data: JsonValue
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgReady {
    pub kind: String,
    pub protocol: ProtocolData
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgClientCapabilities {
    pub kind: String,
    pub id: usize,
    pub capabilities: ProtocolCapabilities
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgDisconnected {
    pub kind: String,
    pub id: usize,
    pub reason: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgMudData {
    pub kind: String,
    pub id: usize,
    pub data: Vec<MudData>
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgClientData {
    pub kind: String,
    pub data: HashMap<usize, ProtocolData>
}


#[derive(Clone, Debug)]
pub struct LinkStub {
    pub conn_id: usize,
    pub addr: SocketAddr,
    pub tls: bool,
    pub tx_link: Sender<Msg2Link>
}

pub struct LinkProtocol<T> {
    conn_id: usize,
    addr: SocketAddr,
    tls: bool,
    conn: WebSocketStream<T>,
    tx_portal: Sender<Msg2Portal>,
    rx_link: Receiver<Msg2Link>,
    running: bool
}

impl<T> LinkProtocol<T> where T: AsyncRead + AsyncWrite + Send + 'static + Unpin + Sync {
    pub fn new(conn_id: usize, conn: WebSocketStream<T>, addr: SocketAddr, tls: bool, tx_portal: Sender<Msg2Portal>, rx_link: Receiver<Msg2Link>) -> Self {

        Self {
            conn_id,
            addr,
            conn,
            tx_portal,
            rx_link,
            tls,
            running: true
        }
    }

    pub async fn run(&mut self) {

        while self.running {
            tokio::select! {
                t_msg = self.conn.next() => {
                    if let Some(msg) = t_msg {
                        match msg {
                            Ok(msg) => {
                                self.process_ws_message(msg).await;
                            },
                            Err(e) => {
                                // Not sure what to do about this yet.
                            }
                        }
                    } else {
                        let _ = self.tx_portal.send(Msg2Portal::LinkDisconnected(self.conn_id, String::from("dunno yet"))).await;
                        self.running = false;
                    }
                },
                p_msg = self.rx_link.recv() => {
                    if let Some(msg) = p_msg {
                        let _ = self.process_link_message(msg).await;
                    }
                }
            }
        }
    }

    async fn process_link_message(&mut self, msg: Msg2Link) {
        match msg {
            Msg2Link::Kill => self.running = false,
            Msg2Link::Replaced => self.running = false,
            Msg2Link::ClientReady(prot) => {
                let out = PortalMsgReady {
                    kind: String::from("client_ready"),
                    protocol: prot.make_data()
                };
                if let Ok(j) = serde_json::to_string(&out) {
                    let _ = self.conn.send(WsMessage::Text(j)).await;
                }
            },
            Msg2Link::ClientData(id, data) => {
                let out = PortalMsgMudData {
                    kind: String::from("client_data"),
                    id,
                    data
                };
                if let Ok(j) = serde_json::to_string(&out) {
                    let _ = self.conn.send(WsMessage::Text(j)).await;
                }
            },
            Msg2Link::ClientDisconnected(id, reason) => {
                let out = PortalMsgDisconnected {
                    kind: String::from("client_disconnected"),
                    id,
                    reason
                };
                if let Ok(j) = serde_json::to_string(&out) {
                    let _ = self.conn.send(WsMessage::Text(j)).await;
                }
            },
            Msg2Link::ClientCapabilities(id, cap) => {
                println!("Link received client cap");
                let out = PortalMsgClientCapabilities {
                    kind: String::from("client_capabilities"),
                    id,
                    capabilities: cap
                };
                if let Ok(j) = serde_json::to_string(&out) {
                    let _ = self.conn.send(WsMessage::Text(j)).await;
                } else {
                    println!("Was there a serialize error?");
                }
            },
            Msg2Link::ClientList(data) => {
                let mut out_data = HashMap::new();
                for (key, value) in data {
                    out_data.insert(key.clone(), value.make_data());
                }
                let out = PortalMsgClientData {
                    kind: String::from("client_list"),
                    data: out_data
                };
                if let Ok(j) = serde_json::to_string(&out) {
                    let _ = self.conn.send(WsMessage::Text(j)).await;
                }
            },
        }
    }

    async fn process_ws_message(&mut self, msg: WsMessage) {
        match msg {
            WsMessage::Binary(b) => {
                // Not sure what to do with this yet.
                println!("Got binary message: {:?}", b);
            },
            WsMessage::Text(s) => {
                if let Ok(j) = JsonValue::from_str(&s) {
                    let _ = self.process_json_message(j).await;
                }
            },
            WsMessage::Close(c) => {
                self.running = false;
                let _ = self.conn.close(None).await;
                let _ = self.tx_portal.send(Msg2Portal::LinkDisconnected(self.conn_id, String::from("dunno"))).await;
            }
            _ => {

            }
        }
    }

    async fn process_json_message(&mut self, msg: JsonValue) {
        print!("Got json message: {}", JsonValue::to_string(&msg));
        let t = &msg["kind"];
        if let Some(s) = t.as_str() {
            match s {
                "client_data" => {
                    if let Ok(p) = serde_json::from_value::<PortalMsgMudData>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id, Msg2PortalFromLink::ClientMessage(p.id.clone(), p.data))).await;
                    }
                },
                _ => {
                }
            }
        }
    }

}