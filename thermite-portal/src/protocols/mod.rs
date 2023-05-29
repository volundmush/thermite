use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use thermite_shared::ProtocolCapabilities;
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

