use std::{
    collections::{HashMap, HashSet},
    io,
    vec::Vec,
    net::SocketAddr,
    time::Duration,
    sync::Arc
};

use tokio::{
    prelude::*,
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

use thermite_net::{Msg2Factory, FactoryLink};
use thermite_telnet::{TelnetCodec, codes as tc, TelnetEvent, TelnetConnectionMode, TelnetConnectionType};
use thermite_util::text::generate_id;

use crate::{Msg2ProtocolManager, Msg2MudProtocol, ProtocolLink,
            ProtocolCapabilities};

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

#[derive(Clone)]
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
    pub fn capabilities(&self) -> ProtocolCapabilities {
        ProtocolCapabilities {
            client_name: self.client_name.clone(),
            client_version: self.client_version.clone(),
            utf8: self.utf8.clone(),
            html: false,
            mxp: self.mxp.clone(),
            msdp: self.msdp.clone(),
            gmcp: self.gmcp.clone(),
            ansi: self.ansi.clone(),
            xterm256: self.xterm256.clone(),
            width: self.width.clone(),
            height: self.height.clone(),
            screen_reader: self.screen_reader.clone()
        }
    }
}

pub struct TelnetProtocol<T> {
    // This serves as a higher-level actor that abstracts a bunch of the lower-level
    // nitty-gritty so the Session doesn't need to deal with it.
    op_state: HashMap<u8, TelnetOptionState>,
    telnet_options: Arc<HashMap<u8, TelnetOption>>,
    conn_id: String,
    addr: SocketAddr,
    tls: bool,
    config: TelnetConfig,
    handshakes_left: HashSet<u8>,
    ttype_count: u8,
    ttype_pending: bool,
    ttype_last: Option<String>,
    conn: Framed<T, TelnetCodec>,
    active: bool,
    sent_link: bool,
    pub tx_protocol: Sender<Msg2MudProtocol>,
    rx_protocol: Receiver<Msg2MudProtocol>,
    tx_manager: Sender<Msg2ProtocolManager>
}


impl<T> TelnetProtocol<T> where T: AsyncRead + AsyncWrite + Send + 'static + Unpin + std::marker::Sync {
    pub fn new(conn_id: String, conn: Framed<T, TelnetCodec>, addr: SocketAddr, tls: bool,
               tx_manager: Sender<Msg2ProtocolManager>, telnet_options: Arc<HashMap<u8, TelnetOption>>,
               op_state: HashMap<u8, TelnetOptionState>) -> Self {

        let (tx_protocol, rx_protocol) = channel(50);

        let mut handshakes_left: HashSet<u8> = Default::default();
        for (k, _v) in op_state.iter() {
            handshakes_left.insert(k.clone());
        }

        Self {
            conn_id,
            addr,
            telnet_options,
            op_state,
            conn,
            config: TelnetConfig::default(),
            handshakes_left,
            tx_protocol,
            rx_protocol,
            tx_manager,
            ttype_count: 0,
            ttype_pending: false,
            ttype_last: None,
            active: false,
            sent_link: false,
            tls
        }
    }

    fn link(&self) -> ProtocolLink {
        ProtocolLink {
            conn_id: self.conn_id.clone(),
            addr: self.addr.clone(),
            tls: self.tls.clone(),
            capabilities: self.config.capabilities(),
            tx_protocol: self.tx_protocol.clone()
        }
    }

    async fn process_handshake(&mut self, op: u8) {
        if self.handshakes_left.contains(&op) {
            self.handshakes_left.remove(&op);
        }
        let _ = self.check_ready().await;
    }

    async fn check_ready(&mut self) {
        if self.active || self.sent_link {
            return;
        }
        if self.handshakes_left.len() == 0 && !self.ttype_pending {
            let _ = self.tx_manager.send(Msg2ProtocolManager::NewProtocol(self.link())).await;
            self.sent_link = true;
        }
    }

    async fn get_ready(&mut self) {
        if self.active || self.sent_link {
            return;
        }
        let _ = self.tx_manager.send(Msg2ProtocolManager::NewProtocol(self.link())).await;
        self.sent_link = true;
    }

    pub async fn run(&mut self, opening: Bytes) {
        
        // Just packing all of this together so it gets sent at once.
        let _ = self.conn.send(TelnetEvent::Data(opening)).await;
        let mut send_chan = self.tx_protocol.clone();

        // Ready a message to fire off quickly for in case
        let ready_task = tokio::spawn(async move {
            time::delay_for(Duration::from_millis(500)).await;
            let _ = send_chan.send(Msg2MudProtocol::GetReady).await;
            ()
        });

        loop {
            tokio::select! {
                t_msg = self.conn.next() => {
                    if let Some(msg) = t_msg {
                        match msg {
                            Ok(msg) => {
                                println!("Got Telnet Event! {:?}", msg);
                                match msg {
                                    TelnetEvent::Line(text) => self.receive_line(text).await,
                                    TelnetEvent::SubNegotiate(op, data) => self.receive_sub(op, data).await,
                                    TelnetEvent::Negotiate(comm, op) => self.receive_negotiate(comm, op).await,
                                    TelnetEvent::NAWS(width, height) => self.receive_naws(width, height).await,
                                    TelnetEvent::TTYPE(text) => self.receive_ttype(text).await,
                                    TelnetEvent::Command(byte) => {},
                                    TelnetEvent::GMCP(command, json) => {},
                                    _ => {}
                                }
                            },
                            Err(e) => {
                                println!("GOT ERROR: {:?}", e);
                                if self.sent_link {
                                    let _ = self.tx_manager.send(Msg2ProtocolManager::ProtocolDisconnected(self.conn_id.clone())).await;
                                }
                                break;
                            }
                        }
                    } else {
                        if self.sent_link {
                                    let _ = self.tx_manager.send(Msg2ProtocolManager::ProtocolDisconnected(self.conn_id.clone())).await;
                                }
                        break;
                    }
                },
                p_msg = self.rx_protocol.recv() => {
                    if let Some(msg) = p_msg {
                        println!("Got Protocol Message: {:?}", msg);
                        match msg {
                            Msg2MudProtocol::Disconnect => {
                                break;
                            },
                            Msg2MudProtocol::Line(text) => {
                                let _ = self.conn.send(TelnetEvent::Line(text)).await;
                            },
                            Msg2MudProtocol::Prompt(text) => {

                            },
                            Msg2MudProtocol::OOB(String, JsonValue) => {

                            },
                            Msg2MudProtocol::MSSP => {

                            },
                            Msg2MudProtocol::Ready => {
                                self.active = true;
                            }
                            Msg2MudProtocol::GetReady => {
                                let _ = self.get_ready().await;
                            }
                        }
                    }
                }
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
                        enable_remote = true;
                        state.remote.enabled = true;
                    }
                },
                tc::WONT => {
                    // The client has refused an option we wanted to enable. Alternatively, it has
                    // disabled an option that was on.
                    if state.remote.negotiating {
                        handshake = op;
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
                        enable_local = true;
                        state.local.enabled = true;
                    }
                },
                tc::DONT => {
                    // The client wants the server to disable Option, or are they are refusing our
                    // desire to do so.
                    if state.local.negotiating {
                        handshake = op;
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
            let _ = self.process_handshake(handshake).await;
        }
    }

    async fn enable_remote(&mut self, op: u8) {
        match op {
            tc::NAWS => self.config.naws = true,
            tc::TTYPE => {
                self.ttype_pending = true;
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
            tc::TTYPE => self.config.ttype = false,
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
            tc::MCCP2 => {
                // Upon getting the OK from the client to use MCCP2, IMMEDIATELY enable the compression.
                
                let _ = self.conn.send(TelnetEvent::SubNegotiate(tc::MCCP2, BytesMut::with_capacity(0).freeze())).await;
                let _ = self.conn.send(TelnetEvent::OutgoingCompression(true)).await;
            }
            _ => {
                
            }
        }
    }

    async fn disable_local(&mut self, op: u8) {

    }

    async fn receive_sub(&mut self, op: u8, data: Bytes) {
        if !self.op_state.contains_key(&op) {
            // Only if we can get a handler, do we want to care about this.
            // All other sub-data is ignored.
            return;
        }
        match op {
            tc::MCCP3 => {
                let _ = self.conn.send(TelnetEvent::IncomingCompression(true)).await;
            },
            _ => {

            }
        }
    }

    async fn receive_line(&mut self, text: String) {
        // No commands will be sent until the Protocol has been recognized by the ProtocolManager.
        println!("RECEIVED LINE: {}", text);
        if self.active {
            let _ = self.tx_manager.send(Msg2ProtocolManager::ProtocolCommand(self.conn_id.clone(), text)).await;
        } else {
            println!("But we're not active.");
        }
    }

    async fn request_ttype(&mut self) {
        let mut data = BytesMut::with_capacity(1);
        data.put_u8(1);
        let _ = self.conn.send(TelnetEvent::SubNegotiate(tc::TTYPE, data.freeze())).await;
    }

    async fn receive_ttype(&mut self, incoming: String) {
        // This is already guaranteed to be good TTYPE data thanks to the TelnetCodec. 'IS' has been
        // stripped, leaving only the following UTF8 text.

        // Ignore if ttype has already been finished.
        if !self.ttype_pending {
            return;
        }

        let upper = incoming.to_uppercase();

        match self.ttype_count {
            0 => {
                self.ttype_last = Some(upper.clone());
                let _ = self.receive_ttype_0(upper.clone()).await;
                self.ttype_count += 1;
                let _ = self.request_ttype().await;
                return;
            },
            1 | 2 => {
                if let Some(last) = self.ttype_last.clone() {
                    if last.eq(&upper) {
                        // This client does not support advanced ttype. Ignore further
                        // calls to TTYPE and consider this complete.
                        self.ttype_pending = false;
                        self.ttype_last = None;
                        let _ = self.check_ready().await;
                    } else {
                        match self.ttype_count {
                            1 => {
                                let _ = self.receive_ttype_1(upper.clone()).await;
                                self.ttype_last = Some(upper.clone());
                                let _ = self.request_ttype().await;
                            },
                            2 => {
                                let _ = self.receive_ttype_2(upper.clone()).await;
                                self.ttype_last = None;
                                self.ttype_pending = false;
                                let _ = self.check_ready().await;
                            }
                            _ => {}
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
    }

    async fn receive_naws(&mut self, width: u16, height: u16) {
        self.config.width = width;
        self.config.height = height;
    }
}

pub struct TelnetProtocolFactory {
    factory_id: String,
    pub tx_factory: Sender<Msg2Factory>,
    rx_factory: Receiver<Msg2Factory>,
    telnet_options: Arc<HashMap<u8, TelnetOption>>,
    telnet_statedef: HashMap<u8, TelnetOptionState>,
    opening_bytes: Bytes,
    tx_manager: Sender<Msg2ProtocolManager>,
    ids: HashSet<String>,
}

impl TelnetProtocolFactory {
    pub fn new(factory_id: String, options: HashMap<u8, TelnetOption>, tx_manager: Sender<Msg2ProtocolManager>) -> Self {
        let (tx_factory, rx_factory) = channel(50);

        // Since this only needs to be done once... we'll clone it from here.
        let mut opstates: HashMap<u8, TelnetOptionState> = Default::default();

        let mut raw_bytes = BytesMut::with_capacity(options.len() * 3);

        for (b, handler) in options.iter() {
            let mut state = TelnetOptionState::default();

            if handler.start_local {
                state.local.negotiating = true;
                raw_bytes.put_u8(tc::IAC);
                raw_bytes.put_u8(tc::WILL);
                raw_bytes.put_u8(b.clone());
            }
            if handler.start_remote {
                state.remote.negotiating = true;
                raw_bytes.put_u8(tc::IAC);
                raw_bytes.put_u8(tc::DO);
                raw_bytes.put_u8(b.clone());
            }
            opstates.insert(b.clone(), state);
        }

        Self {
            factory_id,
            tx_factory,
            rx_factory,
            telnet_options: Arc::new(options),
            telnet_statedef: opstates,
            opening_bytes: raw_bytes.freeze(),
            tx_manager,
            ids: HashSet::default()
        }
    }

    pub fn link(&self) -> FactoryLink {
        FactoryLink {
            factory_id: self.factory_id.clone(),
            tx_factory: self.tx_factory.clone()
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(f_msg) = self.rx_factory.recv().await {
                match f_msg {
                    Msg2Factory::AcceptTLS(stream, addr) => {
                        self.accept(stream, addr, true);
                    },
                    Msg2Factory::AcceptTCP(stream, addr) => {
                        self.accept(stream, addr, false);
                    },
                    Msg2Factory::Kill => {
                        break;
                    }
                }
            }
        }
    }

    fn accept<C>(&mut self, conn: C, addr: SocketAddr, tls: bool)
        where C: AsyncRead + AsyncWrite + Send + 'static + Unpin + std::marker::Sync
    {
        let telnet_codec = Framed::new(conn, TelnetCodec::new(TelnetConnectionType::Server, TelnetConnectionMode::Mud, 8192));
        let gen_id = generate_id(12, &self.ids);
        let conn_id = format!("{}_{}", self.factory_id, gen_id);
        self.ids.insert(gen_id);

        let mut tel_prot = TelnetProtocol::new(conn_id, telnet_codec, addr, tls, self.tx_manager.clone(), self.telnet_options.clone(), self.telnet_statedef.clone());
        let opening = self.opening_bytes.clone();
        let _ = tokio::spawn(async move {tel_prot.run(opening).await;});
    }
}