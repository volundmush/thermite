use std::collections::HashMap;
use thermite_shared::ProtocolCapabilities;
use crate::protocols::link::protocol::LinkStub;
use crate::protocols::{ProtocolLink};
use serde_json::Value as JsonValue;


#[derive(Debug)]
pub enum Msg2MudProtocol {
    Disconnect,
    Prompt(String),
    Text(String),
    GMCP(String, Option<JsonValue>),
    // When a game requests a Mud Server Status Protocol message,
    ServerStatus(Vec<(String, String)>)
}

#[derive(Debug)]
pub enum Msg2PortalFromClient {
    Capabilities(ProtocolCapabilities),
    Line(String),
    GMCP(String, Option<JsonValue>),
}

#[derive(Debug)]
pub enum Msg2PortalFromLink {
    ClientMessage(usize, Msg2MudProtocol),
    ClientDisconnected(usize)
}

#[derive(Debug)]
pub enum Msg2Portal {
    Kill,
    ClientConnected(ProtocolLink),
    ClientDisconnected(usize, String),
    FromClient(usize, Msg2PortalFromClient),
    LinkConnected(LinkStub),
    LinkDisconnected(usize, String),
    FromLink(usize, Msg2PortalFromLink),
}

#[derive(Debug)]
pub enum Msg2Link {
    Kill,
    Replaced,
    ClientReady(ProtocolLink),
    ClientDisconnected(usize, String),
    ClientCapabilities(usize, ProtocolCapabilities),
    ClientLine(usize, String),
    ClientGMCP(usize, String, Option<JsonValue>),
    ClientList(HashMap<usize, ProtocolLink>),
}
