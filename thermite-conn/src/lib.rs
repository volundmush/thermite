pub mod telnet;
pub mod websocket;

use serde_json::Value as JsonValue;
use tokio::sync::mpsc::{Sender, Receiver};

pub enum Msg2MudConnection {
    Disconnect,
    Line(String),
    Prompt(String),
    OOB(String, JsonValue),
    // When a game requests a Mud Server Status Protocol message,
    MSSP,
    Ready
}

pub enum Msg2ConnectionManager {
    NewConnection(ConnectionLink),
    ConnectionCommand(String, String),
    ConnectionDisconnected(String),
}

// This is received by whatever handles connections once they are ready to join the game.
// #TODO: Stick client capabilities in here.
pub struct ConnectionLink {
    pub conn_id: String,
    pub addr: SocketAddr,
    pub tls: bool,
    pub tx_session: Sender<Msg2MudSession>
}