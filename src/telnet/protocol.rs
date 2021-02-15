use std::{
    collections::{HashMap, HashSet},
    vec::Vec,
    net::SocketAddr,
    time::Duration,
    sync::Arc
};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc::{Receiver, Sender, channel},
    time
};

use tokio_util::codec::{Framed};

use bytes::{BytesMut, Bytes, BufMut, Buf};

use futures::{
    sink::{SinkExt},
    stream::{StreamExt}
};

use serde_json::Value as JsonValue;

use crate::{
    telnet::{
        codec::{TelnetCodec, TelnetEvent},
        codes as tc
    },
    net::{Msg2MudProtocol, ProtocolLink, ProtocolCapabilities, Protocol},
    portal::{Msg2Portal, Msg2PortalFromClient}
};


#[derive(Default, Clone)]
pub struct TelnetOptionPerspective {
    pub enabled: bool,
    // Negotiating is true if WE have sent a request.
    pub negotiating: bool
}

#[derive(Default, Clone)]
pub struct TelnetOptionState {
    pub remote: TelnetOptionPerspective,
    pub local: TelnetOptionPerspective,
}

#[derive(Default, Clone)]
pub struct TelnetOption {
    pub allow_local: bool,
    pub allow_remote: bool,
    pub start_local: bool,
    pub start_remote: bool,
}

#[derive(Clone, Debug)]
pub struct TelnetConfig {
    pub client_name: String,
    pub client_version: String,
    pub encoding: String,
    pub utf8: bool,
    pub ansi: bool,
    pub xterm256: bool,
    pub width: u16,
    pub height: u16,
    pub gmcp: bool,
    pub msdp: bool,
    pub mssp: bool,
    pub mxp: bool,
    pub mccp2: bool,
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
    pub truecolor: bool,
    pub mnes: bool
}

impl Default for TelnetConfig {
    fn default() -> Self {
        TelnetConfig {
            client_name: String::from("UNKNOWN"),
            client_version: String::from("UNKNOWN"),
            encoding: String::from("ascii"),
            utf8: false,
            ansi: false,
            xterm256: false,
            width: 78,
            height: 24,
            gmcp: false,
            msdp: false,
            mssp: false,
            mxp: false,
            mccp2: false,
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
            truecolor: false,
            mnes: false
        }
    }
}

impl TelnetConfig {
    fn capabilities(&self) -> ProtocolCapabilities {
        ProtocolCapabilities {
            protocol: Protocol::Telnet,
            client_name: self.client_name.clone(),
            client_version: self.client_version.clone(),
            utf8: self.utf8,
            html: false,
            mxp: self.mxp,
            gmcp: self.gmcp,
            msdp: self.msdp,
            mssp: self.mssp,
            ansi: self.ansi,
            xterm256: self.xterm256,
            width: self.width,
            height: self.height,
            screen_reader: self.screen_reader
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TelnetHandshakes {
    pub local: HashSet<u8>,
    pub remote: HashSet<u8>,
    pub ttype: HashSet<u8>
}

impl TelnetHandshakes {
    pub fn len(&self) -> usize {
        self.local.len() + self.remote.len() + self.ttype.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}


pub struct TelnetProtocol<T> {
    // This serves as a higher-level actor that abstracts a bunch of the lower-level
    // nitty-gritty so the Session doesn't need to deal with it.
    conn_id: String,
    op_state: HashMap<u8, TelnetOptionState>,
    telnet_options: Arc<HashMap<u8, TelnetOption>>,
    addr: SocketAddr,
    tls: bool,
    config: TelnetConfig,
    handshakes_left: TelnetHandshakes,
    ttype_count: u8,
    ttype_last: Option<String>,
    conn: Framed<T, TelnetCodec>,
    active: bool,
    sent_link: bool,
    tx_portal: Sender<Msg2Portal>,
    tx_protocol: Sender<Msg2MudProtocol>,
    rx_protocol: Receiver<Msg2MudProtocol>,
    running: bool,
    app_buffer: BytesMut
}


impl<T> TelnetProtocol<T> where T: AsyncRead + AsyncWrite + Send + 'static + Unpin + std::marker::Sync {
    pub fn new(conn_id: String, conn: Framed<T, TelnetCodec>, addr: SocketAddr, tls: bool, tx_portal: Sender<Msg2Portal>,
               telnet_options: Arc<HashMap<u8, TelnetOption>>, op_state: HashMap<u8, TelnetOptionState>, handshakes_left: TelnetHandshakes) -> Self {

        let (tx_protocol, rx_protocol) = channel(10);
        Self {
            conn_id,
            addr,
            telnet_options,
            op_state,
            conn,
            config: TelnetConfig::default(),
            handshakes_left,
            tx_portal,
            tx_protocol,
            rx_protocol,
            ttype_count: 0,
            ttype_last: None,
            active: false,
            sent_link: false,
            tls,
            running: true,
            app_buffer: BytesMut::with_capacity(1024)
        }
    }

    fn make_link(&self) -> ProtocolLink {
        ProtocolLink {
            conn_id: self.conn_id.clone(),
            addr: self.addr.clone(),
            tls: self.tls,
            capabilities: self.config.capabilities(),
            tx_protocol: self.tx_protocol.clone(),
            json_data: Default::default()
        }
    }

    async fn check_ready(&mut self) {
        if self.active || self.sent_link {
            return;
        }
        if self.handshakes_left.is_empty() {
            let _ = self.tx_portal.send(Msg2Portal::ClientConnected(self.make_link())).await;
            self.sent_link = true;
        }
    }

    async fn get_ready(&mut self) {
        if self.active || self.sent_link {
            return;
        }
        let _ = self.tx_portal.send(Msg2Portal::ClientConnected(self.make_link())).await;
        self.sent_link = true;
    }

    pub async fn run(&mut self, opening: Bytes) {
        // Just packing all of this together so it gets sent at once.
        let _ = self.conn.send(TelnetEvent::Data(opening)).await;
        let send_chan = self.tx_protocol.clone();

        // Ready a message to fire off quickly for in case
        let _ready_task = tokio::spawn(async move {
            time::sleep(Duration::from_millis(500)).await;
            let _ = send_chan.send(Msg2MudProtocol::GetReady).await;
            ()
        });

        while self.running {
            tokio::select! {
                t_msg = self.conn.next() => {
                    if let Some(msg) = t_msg {
                        match msg {
                            Ok(msg) => {
                                let _ = self.process_telnet_event(msg).await;
                            },
                            Err(e) => {
                                // Not sure what to do about this yet.
                            }
                        }
                    } else {
                        let _ = self.tx_portal.send(Msg2Portal::ClientDisconnected(self.conn_id.clone(), String::from("dunno yet"))).await;
                        self.running = false;
                    }
                },
                p_msg = self.rx_protocol.recv() => {
                    if let Some(msg) = p_msg {
                        let _ = self.process_protocol_message(msg).await;
                    }
                }
            }
        }
    }

    async fn process_telnet_event(&mut self, msg: TelnetEvent) {
        match msg {
            TelnetEvent::SubNegotiate(op, data) => self.receive_sub(op, data).await,
            TelnetEvent::Negotiate(comm, op) => self.receive_negotiate(comm, op).await,
            TelnetEvent::Command(byte) => {},
            TelnetEvent::Data(data) => {
                let _ = self.process_telnet_data(data).await;
            }
        }
    }

    async fn process_telnet_data(&mut self, data: Bytes) {
        self.app_buffer.put(data);
        while let Some(ipos) = self.app_buffer.as_ref().iter().position(|b| b == &tc::LF) {
            let cmd = self.app_buffer.split_to(ipos);
            if let Ok(s) = String::from_utf8(cmd.to_vec()) {
                let _ = self.handle_user_command(s.trim().to_string()).await;
            }
            self.app_buffer.advance(1);
        }
    }

    async fn handle_user_command(&mut self, cmd: String) {
        let _ = self.tx_portal.send(Msg2Portal::FromClient(self.conn_id.clone(), Msg2PortalFromClient::Line(cmd))).await;
    }

    async fn process_protocol_message(&mut self, msg: Msg2MudProtocol) {
        match msg {
            Msg2MudProtocol::Disconnect => {
                self.running = false;
            },
            Msg2MudProtocol::Line(l) => {
                if l.ends_with("\n") {
                    let _ = self.conn.send(TelnetEvent::Data(Bytes::from(l))).await;
                } else {
                    let _ = self.conn.send(TelnetEvent::Data(Bytes::from(l + "\n"))).await;
                }
            },
            Msg2MudProtocol::Prompt(text) => {
                let _ = self.conn.send(TelnetEvent::Data(Bytes::from(text))).await;
            },
            Msg2MudProtocol::GMCP(cmd, data) => {
                //let _ = self.conn.send(TelnetEvent::GMCP(cmd, data)).await;
            },
            Msg2MudProtocol::ServerStatus(data) => {
                //let _ = self.conn.send(TelnetEvent::MSSP(data)).await;
            },
            Msg2MudProtocol::GetReady => {
                let _ = self.get_ready().await;
            },
            Msg2MudProtocol::Lines(data) => {
                for l in data {
                    if l.ends_with("\n") {
                        let _ = self.conn.send(TelnetEvent::Data(Bytes::from(l))).await;
                    } else {
                        let _ = self.conn.send(TelnetEvent::Data(Bytes::from(l + "\n"))).await;
                    }
                }
            },
            Msg2MudProtocol::Text(s) => {
                let _ = self.conn.send(TelnetEvent::Data(Bytes::from(s))).await;
            }
        }
    }

    async fn receive_negotiate(&mut self, command: u8, op: u8) {
        // This means we received an IAC will/wont/do/dont...
        let mut handshake: u8 = 0;
        let mut enable_local = false;
        let mut disable_local = false;
        let mut enable_remote = false;
        let mut disable_remote = false;
        let mut handshake_remote: u8 = 0;
        let mut handshake_local: u8 = 0;
        let mut respond: u8 = 0;

        if let Some(state) = self.op_state.get_mut(&op) {
            // We DO have a handler for this option... that means we support it!

            match command {
                tc::WILL => {
                    // The remote host has sent a WILL. They either want to Locally-Enable op, or are
                    // doing so at our request.
                    if !state.remote.enabled {
                        if state.remote.negotiating {
                            state.remote.negotiating = false;
                        }
                        else {
                            respond = tc::DO;
                        }
                        handshake = op;
                        handshake_remote = op;
                        enable_remote = true;
                        state.remote.enabled = true;
                    }
                },
                tc::WONT => {
                    // The client has refused an option we wanted to enable. Alternatively, it has
                    // disabled an option that was on.
                    if state.remote.negotiating {
                        handshake = op;
                        handshake_remote = op;
                    }
                    state.remote.negotiating = false;
                    if state.remote.enabled {
                        disable_remote = true;
                        state.remote.enabled = false;
                    }
                },
                tc::DO => {
                    // The client wants the Server to enable Option, or they are acknowledging our
                    // desire to do so.
                    if !state.local.enabled {
                        if state.local.negotiating {
                            state.local.negotiating = false;
                        }
                        else {
                            respond = tc::WILL;
                        }
                        handshake = op;
                        handshake_local = op;
                        enable_local = true;
                        state.local.enabled = true;
                    }
                },
                tc::DONT => {
                    // The client wants the server to disable Option, or are they are refusing our
                    // desire to do so.
                    if state.local.negotiating {
                        handshake = op;
                        handshake_local = op;
                    }
                    state.local.negotiating = false;
                    if state.local.enabled {
                        disable_local = true;
                        state.local.enabled = false
                    }
                },
                _ => {
                    // This cannot actually happen.
                }
            }
        } else {
            // We do not have a handler for this option, whatever it is... do not support.
            respond = match command {
                tc::WILL => tc::DONT,
                tc::DO => tc::WONT,
                _ => 0
            };
        }
        
        if respond > 0 {
            let _ = self.conn.send(TelnetEvent::Negotiate(respond, op)).await;
        }
        if handshake_local > 0 {
            self.handshakes_left.local.remove(&handshake_local);
        }
        if handshake_remote > 0 {
            self.handshakes_left.remote.remove(&handshake_remote);
        }
        if enable_local {
            let _ = self.enable_local(op).await;
        }
        if disable_local {
            let _ = self.disable_local(op).await;
        }
        if enable_remote {
            let _ = self.enable_remote(op).await;
        }
        if disable_remote {
            let _ = self.disable_remote(op).await;
        }
        if handshake > 0 {
            let _ = self.check_ready().await;
        }
    }

    async fn enable_remote(&mut self, op: u8) {
        match op {
            tc::NAWS => self.config.naws = true,
            tc::TTYPE => {
                self.request_ttype().await;
            },
            tc::LINEMODE => self.config.linemode = true,
            _ => {
                // Whatever this option is.. well, whatever.
            }
        }
    }

    async fn disable_remote(&mut self, op: u8) {
        match op {
            tc::NAWS => {
                self.config.naws = false;
                self.config.width = 78;
                self.config.height = 24;
            }
            tc::TTYPE => {
                self.config.ttype = false;
                self.handshakes_left.ttype.clear();
            },
            tc::LINEMODE => self.config.linemode = false,
            _ => {
                // Whatever this option is.. well, whatever.
            }
        }
    }

    async fn enable_local(&mut self, op: u8) {
        match op {
            tc::SGA => {
                self.config.sga = true;
            },
            _ => {
                
            }
        }
    }

    async fn disable_local(&mut self, op: u8) {
        match op {
            tc::SGA => {
                self.config.sga = false;
            },
            _ => {

            }
        }
    }

    async fn receive_sub(&mut self, op: u8, mut data: Bytes) {
        if !self.op_state.contains_key(&op) {
            // Only if we can get a handler, do we want to care about this.
            // All other sub-data is ignored.
            return;
        }

        match op {
            tc::NAWS => {
                let _ = self.receive_naws(data).await;
            },
            tc::TTYPE => {
                let _ = self.receive_ttype(data).await;
            }
            _ => {}
        }

    }

    async fn request_ttype(&mut self) {
        let mut data = BytesMut::with_capacity(1);
        data.put_u8(1);
        let _ = self.conn.send(TelnetEvent::SubNegotiate(tc::TTYPE, data.freeze())).await;
    }

    async fn receive_ttype(&mut self, mut data: Bytes) {

        if data.len() < 2 {
            return
        }

        if self.handshakes_left.ttype.is_empty() {
            return;
        }

        if data[0] != 0 {
            return;
        }

        data.advance(1);

        if let Ok(s) = String::from_utf8(data.to_vec()) {
            let upper = s.trim().to_uppercase();

            match self.ttype_count {
                0 => {
                    self.ttype_last = Some(upper.clone());
                    let _ = self.receive_ttype_0(upper.clone()).await;
                    self.ttype_count += 1;
                    self.handshakes_left.ttype.remove(&0);
                    let _ = self.request_ttype().await;
                    return;
                },
                1 | 2 => {
                    if let Some(last) = self.ttype_last.clone() {
                        if last.eq(&upper) {
                            // This client does not support advanced ttype. Ignore further
                            // calls to TTYPE and consider this complete.
                            self.handshakes_left.ttype.clear();
                            self.ttype_last = None;
                            let _ = self.check_ready().await;
                        } else {
                            match self.ttype_count {
                                1 => {
                                    let _ = self.receive_ttype_1(upper.clone()).await;
                                    self.ttype_last = Some(upper.clone());
                                },
                                2 => {
                                    let _ = self.receive_ttype_2(upper.clone()).await;
                                    self.ttype_last = None;
                                    self.handshakes_left.ttype.clear();
                                }
                                _ => {}
                            }
                            if self.handshakes_left.ttype.is_empty() {
                                let _ = self.check_ready().await;
                            }
                        }
                    }
                    return;
                }
                _ => {
                    // This shouldn't happen.
                }
            }
        }


    }

    async fn receive_ttype_0(&mut self, data: String) {
        // The first TTYPE receives the name of the client.
        // version might also be in here as a second word.
        if data.contains(" ") {
            let results: Vec<&str> = data.splitn(1, " ").collect();
            self.config.client_name = String::from(results[0]);
            self.config.client_version = String::from(results[1]);
        } else {
            self.config.client_name = data;
        }

        // Now that the name and version (may be UNKNOWN) are set... we can deduce capabilities.
        let mut extra_check = false;
        match self.config.client_name.as_str() {
            "ATLANTIS" => {
                self.config.xterm256 = true;
                self.config.ansi = true;
            },
            "CMUD" => {
                self.config.xterm256 = true;
                self.config.ansi = true;
            },
            "KILDCLIENT" => {
                self.config.xterm256 = true;
                self.config.ansi = true;
            },
            "MUDLET" => {
                self.config.xterm256 = true;
                self.config.ansi = true;
            },
            "MUSHCLIENT" => {
                self.config.xterm256 = true;
                self.config.ansi = true;
            },
            "PUTTY" => {
                self.config.xterm256 = true;
                self.config.ansi = true;
            },
            "BEIP" => {
                self.config.xterm256 = true;
                self.config.ansi = true;
            },
            "POTATO" => {
                self.config.xterm256 = true;
                self.config.ansi = true;
            },
            "TINYFUGUE" => {
                self.config.xterm256 = true;
                self.config.ansi = true;
            }
            _ => {
                extra_check = true;
            }
        }
        if extra_check {
            if self.config.client_name.starts_with("XTERM") || self.config.client_name.ends_with("-256COLOR") {
                self.config.xterm256 = true;
                self.config.ansi = true;
            }
        }
    }

    async fn receive_ttype_1(&mut self, data: String) {
        if data.starts_with("XTERM") || data.ends_with("-256COLOR") {
            self.config.xterm256 = true;
            self.config.ansi = true;
        }
        self.handshakes_left.ttype.remove(&1);
    }

    async fn receive_ttype_2(&mut self, data: String) {
        if !data.starts_with("MTTS ") {
            return;
        }
        let results: Vec<&str> = data.splitn(2, " ").collect();
        let value = String::from(results[1]);
        let mtts: usize = value.parse().unwrap_or(0);
        if mtts == 0 {
            return;
        }
        if (1 & mtts) == 1 {
            self.config.ansi = true;
        }
        if (2 & mtts) == 2 {
            self.config.vt100 = true;
        }
        if (4 & mtts) == 4 {
            self.config.utf8 = true;
        }
        if (8 & mtts) == 8 {
            self.config.xterm256 = true;
        }
        if (16 & mtts) == 16 {
            self.config.mouse_tracking = true;
        }
        if (32 & mtts) == 32 {
            self.config.osc_color_palette = true;
        }
        if (64 & mtts) == 64 {
            self.config.screen_reader = true;
        }
        if (128 & mtts) == 128 {
            self.config.proxy = true;
        }
        if (256 & mtts) == 256 {
            self.config.truecolor = true;
        }
        if (512 & mtts) == 512 {
            self.config.mnes = true;
        }
        self.handshakes_left.ttype.remove(&2);
    }

    async fn receive_naws(&mut self, mut data: Bytes) {
        if data.len() >= 4 {
            let old_width = self.config.width;
            let old_height = self.config.height;
            self.config.width = data.get_u16();
            self.config.height = data.get_u16();

            if self.config.width != old_width || self.config.height != old_height {
                let _ = self.update_capabilities().await;
            }
        }
    }

    async fn update_capabilities(&mut self) {
        if self.active {
            let _ = self.tx_portal.send(Msg2Portal::FromClient(self.conn_id.clone(), Msg2PortalFromClient::Capabilities(self.config.capabilities()))).await;
        }
    }
}