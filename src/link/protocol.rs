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

use crate::{
    portal::{Msg2Portal, Msg2PortalFromLink},
    net::{Msg2MudProtocol, ProtocolCapabilities, ProtocolLink, ProtocolData}
};

use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Message as WsMessage;

pub enum Msg2Link {
    Kill,
    Replaced,
    ClientReady(ProtocolLink),
    ClientDisconnected(String, String),
    ClientCapabilities(String, ProtocolCapabilities),
    ClientLines(String, Vec<String>),
    ClientLine(String, String),
    ClientGMCP(String, String, JsonValue),
    ClientList(HashMap<String, ProtocolLink>),
    PortalJson(JsonValue),
    ClientJson(String, JsonValue)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionLines {
    pub kind: String,
    pub id: String,
    pub lines: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionLine {
    pub kind: String,
    pub id: String,
    pub line: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionText {
    pub kind: String,
    pub id: String,
    pub text: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionGMCP {
    pub kind: String,
    pub id: String,
    pub gmcp_cmd: String,
    pub gmcp_data: JsonValue
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionMSSP {
    pub kind: String,
    pub id: String,
    pub mssp: Vec<(String, String)>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionPrompt {
    pub kind: String,
    pub id: String,
    pub prompt: String
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionDisconnect {
    pub kind: String,
    pub id: String,
    pub reason: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionJson {
    pub kind: String,
    pub id: String,
    pub data: JsonValue
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgRequestCapabilities {
    pub kind: String,
    pub id: String
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
    pub id: String,
    pub capabilities: ProtocolCapabilities
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgDisconnected {
    pub kind: String,
    pub id: String,
    pub reason: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgLines {
    pub kind: String,
    pub id: String,
    pub lines: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgLine {
    pub kind: String,
    pub id: String,
    pub line: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgGMCP {
    pub kind: String,
    pub id: String,
    pub gmcp_cmd: String,
    pub gmcp_data: JsonValue
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgClientData {
    pub kind: String,
    pub data: HashMap<String, ProtocolData>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgMSSPRequest {
    pub kind: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgJson {
    pub kind: String,
    pub data: JsonValue
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortalMsgClientJson {
    pub kind: String,
    pub id: String,
    pub data: JsonValue
}


#[derive(Clone, Debug)]
pub struct LinkStub {
    pub conn_id: String,
    pub addr: SocketAddr,
    pub tls: bool,
    pub tx_link: Sender<Msg2Link>
}

pub struct LinkProtocol<T> {
    conn_id: String,
    addr: SocketAddr,
    tls: bool,
    conn: WebSocketStream<T>,
    tx_portal: Sender<Msg2Portal>,
    rx_link: Receiver<Msg2Link>,
    running: bool,
    json_data: JsonValue
}

impl<T> LinkProtocol<T> where T: AsyncRead + AsyncWrite + Send + 'static + Unpin + Sync {
    pub fn new(conn_id: String, conn: WebSocketStream<T>, addr: SocketAddr, tls: bool, tx_portal: Sender<Msg2Portal>, rx_link: Receiver<Msg2Link>) -> Self {

        Self {
            conn_id,
            addr,
            conn,
            tx_portal,
            rx_link,
            tls,
            running: true,
            json_data: Default::default()
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
                        let _ = self.tx_portal.send(Msg2Portal::LinkDisconnected(self.conn_id.clone(), String::from("dunno yet"))).await;
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
            Msg2Link::ClientLines(id, lines) => {
                let out = PortalMsgLines {
                    kind: String::from("client_lines"),
                    id,
                    lines
                };
                if let Ok(j) = serde_json::to_string(&out) {
                    let _ = self.conn.send(WsMessage::Text(j)).await;
                }
            },
            Msg2Link::ClientLine(id, line) => {
                let out = PortalMsgLine {
                    kind: String::from("client_line"),
                    id,
                    line
                };
                if let Ok(j) = serde_json::to_string(&out) {
                    let _ = self.conn.send(WsMessage::Text(j)).await;
                }
            },
            Msg2Link::ClientGMCP(id, cmd, data) => {
                let out = PortalMsgGMCP {
                    kind: String::from("client_gmcp"),
                    id,
                    gmcp_cmd: cmd,
                    gmcp_data: data
                };
                if let Ok(j) = serde_json::to_string(&out) {
                    let _ = self.conn.send(WsMessage::Text(j)).await;
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
            Msg2Link::PortalJson(j) => {
                let out = PortalMsgJson {
                    kind: String::from("portal_json"),
                    data: j
                };
                if let Ok(j) = serde_json::to_string(&out) {
                    let _ = self.conn.send(WsMessage::Text(j)).await;
                }
            },
            Msg2Link::ClientJson(id, j) => {
                let out = PortalMsgClientJson {
                    kind: String::from("client_json"),
                    id,
                    data: j
                };
                if let Ok(j) = serde_json::to_string(&out) {
                    let _ = self.conn.send(WsMessage::Text(j)).await;
                }
            }
        }
    }

    async fn process_ws_message(&mut self, msg: WsMessage) {
        match msg {
            WsMessage::Text(s) => {
                if let Ok(j) = JsonValue::from_str(&s) {
                    let _ = self.process_json_message(j).await;
                }
            },
            WsMessage::Close(c) => {
                self.running = false;
                let _ = self.conn.close(None).await;
                let _ = self.tx_portal.send(Msg2Portal::LinkDisconnected(self.conn_id.clone(), String::from("dunno"))).await;
            }
            _ => {

            }
        }
    }

    async fn process_json_message(&mut self, msg: JsonValue) {
        let t = &msg["kind"];
        if let Some(s) = t.as_str() {
            match s {
                "session_lines" => {
                    if let Ok(p) = serde_json::from_value::<ServerMsgSessionLines>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::ClientMessage(p.id.clone(), Msg2MudProtocol::Lines(p.lines)))).await;
                    }
                },
                "session_line" => {
                    if let Ok(p) = serde_json::from_value::<ServerMsgSessionLine>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::ClientMessage(p.id.clone(), Msg2MudProtocol::Line(p.line)))).await;
                    }
                },
                "session_text" => {
                    if let Ok(p) = serde_json::from_value::<ServerMsgSessionText>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::ClientMessage(p.id.clone(), Msg2MudProtocol::Text(p.text)))).await;
                    }
                },
                "session_gmcp" => {
                    if let Ok(p) = serde_json::from_value::<ServerMsgSessionGMCP>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::ClientMessage(p.id.clone(), Msg2MudProtocol::GMCP(p.gmcp_cmd.clone(), p.gmcp_data)))).await;
                    }
                },
                "session_mssp" => {
                    if let Ok(p) = serde_json::from_value::<ServerMsgSessionMSSP>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::ClientMessage(p.id.clone(), Msg2MudProtocol::ServerStatus(p.mssp)))).await;
                    }
                },
                "session_prompt" => {
                    if let Ok(p) = serde_json::from_value::<ServerMsgSessionPrompt>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::ClientMessage(p.id.clone(), Msg2MudProtocol::Prompt(p.prompt)))).await;
                    }
                },
                "session_json" => {
                    if let Ok(p) = serde_json::from_value::<ServerMsgSessionJson>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::SetClientJson(p.id.clone(), p.data))).await;
                    }
                },
                "session_request_json" => {
                    if let Ok(p) = serde_json::from_value::<ServerMsgSessionJson>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::RequestClientJson(p.id))).await;
                    }
                },
                "session_request_capabilities" => {
                    if let Ok(p) = serde_json::from_value::<ServerMsgRequestCapabilities>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::RequestClientCapabilities(p.id))).await;
                    }
                },
                "server_request_clients" => {
                    let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::RequestClientList)).await;
                },
                "server_json" => {
                    if let Ok(p) = serde_json::from_value::<ServerMsgJson>(msg.clone()) {
                        let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::SetPortalJson(p.data))).await;
                    }
                },
                "server_request_json" => {
                    let _ = self.tx_portal.send(Msg2Portal::FromLink(self.conn_id.clone(), Msg2PortalFromLink::RequestPortalJson)).await;
                },
                _ => {
                }
            }
        }
    }

}