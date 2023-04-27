use std::collections::HashMap;
use crate::protocols::link::protocol::LinkStub;
use crate::protocols::{ProtocolCapabilities, ProtocolLink};
use serde_json::Value as JsonValue;

#[derive(Debug)]
pub enum Msg2MudProtocol {
    Disconnect,
    Line(String),
    Lines(Vec<String>),
    Prompt(String),
    Text(String),
    GMCP(String, JsonValue),
    // When a game requests a Mud Server Status Protocol message,
    ServerStatus(Vec<(String, String)>)
}


#[derive(Debug)]
pub enum Msg2PortalFromClient {
    Capabilities(ProtocolCapabilities),
    Line(String),
    Lines(Vec<String>),
    GMCP(usize, JsonValue),
}

#[derive(Debug)]
pub enum Msg2PortalFromLink {
    ClientMessage(usize, Msg2MudProtocol),
    ClientDisconnected(usize),
    RequestClientCapabilities(usize),
    RequestClientList
}

#[derive(Debug)]
pub enum Msg2Portal {
    Kill,
    ClientConnected(ProtocolLink),
    ClientDisconnected(usize, String),
    FromClient(usize, Msg2PortalFromClient),
    LinkConnected(LinkStub),
    LinkDisconnected(String, String),
    FromLink(String, Msg2PortalFromLink),
}


#[derive(Debug)]
pub enum Msg2Link {
    Kill,
    Replaced,
    ClientReady(ProtocolLink),
    ClientDisconnected(usize, String),
    ClientCapabilities(usize, ProtocolCapabilities),
    ClientLines(usize, Vec<String>),
    ClientLine(usize, String),
    ClientGMCP(usize, String, JsonValue),
    ClientList(HashMap<String, ProtocolLink>),
}