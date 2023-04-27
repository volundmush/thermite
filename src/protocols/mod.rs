use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;
use crate::msg::Msg2MudProtocol;

use serde::{Serialize, Deserialize};

pub mod link;
pub mod telnet;
// pub mod websocket;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolData {
    pub id: usize,
    pub addr: SocketAddr,
    pub capabilities: ProtocolCapabilities
}

// This is received by whatever handles connections once they are ready to join the game.
#[derive(Debug, Clone)]
pub struct ProtocolLink {
    pub conn_id: usize,
    pub addr: SocketAddr,
    pub capabilities: ProtocolCapabilities,
    pub tx_protocol: Sender<Msg2MudProtocol>
}

impl ProtocolLink {
    pub fn make_data(&self) -> ProtocolData {
        ProtocolData {
            id: self.conn_id.clone(),
            addr: self.addr.clone(),
            capabilities: self.capabilities.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Protocol {
    Telnet = 0,
    WebSocket = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Color {
    NoColor = 0,
    Ansi = 1,
    Xterm256 = 2,
    TrueColor = 3
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolCapabilities {
    pub protocol: Protocol,
    pub encryption: bool,
    pub client_name: String,
    pub client_version: String,
    pub encoding: String,
    pub utf8: bool,
    pub color: Color,
    pub width: u16,
    pub height: u16,
    pub gmcp: bool,
    pub msdp: bool,
    pub mssp: bool,
    pub mxp: bool,
    pub mccp2: bool,
    pub mccp2_enable: bool,
    pub mccp3: bool,
    pub mccp3_enable: bool,
    pub ttype: bool,
    pub naws: bool,
    pub sga: bool,
    pub linemode: bool,
    pub force_endline: bool,
    pub oob: bool,
    pub tls: bool,
    pub screen_reader: bool,
    pub mouse_tracking: bool,
    pub vt100: bool,
    pub osc_color_palette: bool,
    pub proxy: bool,
    pub mnes: bool
}

impl Default for ProtocolCapabilities {
    fn default() -> Self {
        Self {
            protocol: Protocol::Telnet,
            width: 78,
            height: 24,
            client_name: "UNKNOWN".to_string(),
            client_version: "UNKNOWN".to_string(),
            ..Default::default()
        }
    }
}