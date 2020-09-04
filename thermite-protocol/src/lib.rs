use serde_json::Value as JsonValue;
use tokio::sync::mpsc::{Sender};
use std::net::SocketAddr;

pub mod telnet;
pub mod websocket;

#[derive(Debug)]
pub enum Msg2MudProtocol {
    Disconnect,
    Line(String),
    Prompt(String),
    OOB(String, JsonValue),
    // When a game requests a Mud Server Status Protocol message,
    MSSP,
    GetReady,
    Ready
}

#[derive(Debug)]
pub enum Msg2ProtocolManager {
    NewProtocol(ProtocolLink),
    ProtocolCommand(String, String),
    ProtocolDisconnected(String),
}

#[derive(Debug)]
pub struct ProtocolCapabilities {
    pub client_name: String,
    pub client_version: String,
    pub utf8: bool,
    pub html: bool,
    pub mxp: bool,
    pub gmcp: bool,
    pub msdp: bool,
    pub ansi: bool,
    pub xterm256: bool,
    pub width: u16,
    pub height: u16,
    pub screen_reader: bool,
}

// This is received by whatever handles connections once they are ready to join the game.
#[derive(Debug)]
pub struct ProtocolLink {
    pub conn_id: String,
    pub addr: SocketAddr,
    pub tls: bool,
    pub capabilities: ProtocolCapabilities,
    pub tx_protocol: Sender<Msg2MudProtocol>
}