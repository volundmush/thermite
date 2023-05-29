use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionLines {
    pub kind: String,
    pub id: usize,
    pub lines: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionLine {
    pub kind: String,
    pub id: usize,
    pub line: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionText {
    pub kind: String,
    pub id: usize,
    pub text: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionGMCP {
    pub kind: String,
    pub id: usize,
    pub gmcp_cmd: String,
    pub gmcp_data: Option<JsonValue>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionMSSP {
    pub kind: String,
    pub id: usize,
    pub mssp: Vec<(String, String)>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionPrompt {
    pub kind: String,
    pub id: usize,
    pub prompt: String
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgSessionDisconnect {
    pub kind: String,
    pub id: usize,
    pub reason: String
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsgRequestCapabilities {
    pub kind: String,
    pub id: usize
}