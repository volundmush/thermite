use std::collections::HashMap;
use crate::protocols::link::protocol::LinkStub;
use crate::protocols::{ProtocolCapabilities, ProtocolLink, MudData};
use serde_json::Value as JsonValue;

#[derive(Debug)]
pub enum Msg2MudProtocol {
    Disconnect,
    Data(Vec<MudData>)
}

#[derive(Debug)]
pub enum Msg2PortalFromClient {
    Capabilities(ProtocolCapabilities),
    Data(Vec<MudData>)
}

#[derive(Debug)]
pub enum Msg2PortalFromLink {
    ClientMessage(usize, Vec<MudData>),
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
    Broadcast(String)
}

#[derive(Debug)]
pub enum Msg2Link {
    Kill,
    Replaced,
    ClientReady(ProtocolLink),
    ClientDisconnected(usize, String),
    ClientCapabilities(usize, ProtocolCapabilities),
    ClientData(usize, Vec<MudData>),
    ClientList(HashMap<usize, ProtocolLink>),
}
