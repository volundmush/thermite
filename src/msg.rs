use std::{
    collections::{HashMap},
    net::SocketAddr,
};
use tokio::{
    sync::oneshot,
    net::TcpStream
};
use tokio_rustls::TlsStream;

use crate::{
    net::{ProtocolCapabilities, ProtocolLink, ConnectResponse}
};

#[derive(Debug)]
pub enum Msg2Game {
    NewProtocol(ProtocolLink, oneshot::Sender<ConnectResponse>),
    ProtocolCommand(String, String),
    ProtocolGMCP(String, String, serde_json::Value),
    ProtocolDisconnected(String),
    UpdateCapabilities(String, ProtocolCapabilities),
    GameKick(String),
    MSSP(oneshot::Sender<HashMap<String, String>>),
}

