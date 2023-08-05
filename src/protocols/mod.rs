use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use crate::msg::Msg2MudProtocol;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

pub mod link;
pub mod telnet;
pub mod websocket;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MudData {
    pub cmd: String,
    pub args: Vec<JsonValue>,
    pub kwargs: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolData {
    pub id: usize,
    pub capabilities: ProtocolCapabilities
}

// This is received by whatever handles connections once they are ready to join the game.
#[derive(Debug, Clone)]
pub struct ProtocolLink {
    pub conn_id: usize,
    pub capabilities: ProtocolCapabilities,
    pub tx_protocol: Sender<Msg2MudProtocol>
}

impl ProtocolLink {
    pub fn make_data(&self) -> ProtocolData {
        ProtocolData {
            id: self.conn_id.clone(),
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
    Standard = 1,
    Xterm256 = 2,
    TrueColor = 3
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolCapabilities {
    pub protocol: Protocol,
    pub encryption: bool,
    pub client_name: String,
    pub client_version: String,
    pub host_address: String,
    pub host_port: u16,
    pub host_names: Vec<String>,
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
    pub mccp3: bool,
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
            host_address: "UNKNOWN".to_string(),
            host_port: 0,
            host_names: vec![],
            encoding: Default::default(),
            utf8: false,
            color: Color::NoColor,
            encryption: false,
            gmcp: false,
            msdp: false,
            mssp: false,
            mxp: false,
            mccp2: false,
            mccp3: false,
            ttype: false,
            naws: false,
            sga: false,
            linemode: false,
            force_endline: false,
            oob: false,
            tls: false,
            screen_reader: false,
            mouse_tracking: false,
            vt100: false,
            osc_color_palette: false,
            proxy: false,
            mnes: false
        }
    }
}