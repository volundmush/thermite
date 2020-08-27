use tokio::{
    prelude::*,
    task::JoinHandle,
    sync::mpsc::{Receiver, Sender, channel},
    net::{TcpListener, TcpStream}
};

use tokio_util::codec::{Encoder, Decoder, Framed};


use std::{
    collections::HashMap,
    iter,
    io,
    vec::Vec,
    net::SocketAddr,
    convert::TryInto
};

use flate2::{
    Compression,
    write::{ZlibEncoder, ZlibDecoder}
};

use bytes::{BytesMut, Buf, BufMut};

use futures::{
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt}
};

use crate::conn::{
    Msg2Portal,
    Msg2SessionManager,
    Msg2Protocol,
    Msg2Session,
    ClientInfo, ClientCapabilities
};

pub mod tc {
    pub const NULL: u8 = 0;
    pub const BEL: u8 = 7;
    pub const CR: u8 = 13;
    pub const LF: u8 = 10;
    pub const SGA: u8 = 3;
    pub const TELOPT_EOR: u8 = 25;
    pub const NAWS: u8 = 31;
    pub const LINEMODE: u8 = 34;
    pub const EOR: u8 = 239;
    pub const SE: u8 = 240;
    pub const NOP: u8 = 241;
    pub const GA: u8 = 249;
    pub const SB: u8 = 250;
    pub const WILL: u8 = 251;
    pub const WONT: u8 = 252;
    pub const DO: u8 = 253;
    pub const DONT: u8 = 254;
    pub const IAC: u8 = 255;

    // The following are special MUD specific protocols.

    // MUD eXtension Protocol
    pub const MXP: u8 = 91;

    // Mud Server Status Protocol
    pub const MSSP: u8 = 70;

    // Compression
    // pub const MCCP1: u8 = 85 - this is deprecrated
    // NOTE: MCCP2 and MCCP3 is currently disabled.
    pub const MCCP2: u8 = 86;
    pub const MCCP3: u8 = 87;

    // GMCP - Generic Mud Communication Protocol
    pub const GMCP: u8 = 201;

    // MSDP - Mud Server Data Protocol
    pub const MSDP: u8 = 69;

    // TTYPE - Terminal Type
    pub const TTYPE: u8 = 24;
}


pub enum TelnetState {
    Data,
    Sub(u8),
}

// Line was definitely a line.
// Data byte by byte application data... bad news for me.
#[derive(Clone)]
pub enum TelnetReceive {
    Line(Vec<u8>),
    Data(u8),
    Will(u8),
    Wont(u8),
    Do(u8),
    Dont(u8),
    Sub((u8, Vec<u8>))
}

pub enum TelnetSend {
    Data(Vec<u8>),
    Line(Vec<u8>),
    Prompt(Vec<u8>),
    Sub((u8, Vec<u8>)),
    Command((u8, u8)),
    RawBytes(Vec<u8>)
}

pub enum IacSection {
    Command((u8, u8)),
    IAC,
    Pending,
    Error,
    SE
}

pub struct TelnetCodec {
    sub_data: Vec<u8>,
    app_data: Vec<u8>,
    line_mode: bool,
    state: TelnetState,
    //mccp2: bool,
    //mccp3: bool
}

pub enum SubState {
    Data,
    Escaped
}

impl TelnetCodec {
    pub fn new(line_mode: bool) -> Self {
        TelnetCodec {
            app_data: Vec::with_capacity(1024),
            sub_data: Vec::with_capacity(1024),
            line_mode,
            state: TelnetState::Data,
            //mccp2: false,
            //mccp3: false
        }
    }
}

impl Decoder for TelnetCodec {
    type Item = TelnetReceive;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // should put something here about checking for WAY TOO MUCH BYTES... and kicking if
        // abuse is detected.

        loop {
            if src.is_empty() {
                return Ok(None);
            }


            if src[0] == tc::IAC {
                let (res, consume) = self.try_parse_iac(src.bytes());
                src.advance(consume);

                match res {
                    IacSection::Error => {

                    },
                    IacSection::Command((comm, op)) => {
                        match comm {
                            tc::WILL => return Ok(Some(TelnetReceive::Will(op))),
                            tc::WONT => return Ok(Some(TelnetReceive::Wont(op))),
                            tc::DO => return Ok(Some(TelnetReceive::Do(op))),
                            tc::DONT => return Ok(Some(TelnetReceive::Dont(op))),
                            tc::SB => {
                                match self.state {
                                    TelnetState::Data => {
                                        self.state = TelnetState::Sub(op);
                                    },
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    },
                    // this occurs if the IAC is not complete.
                    IacSection::Pending => return Ok(None),
                    IacSection::IAC => {
                        self.app_data.push(tc::IAC);
                    },
                    IacSection::SE => {
                        match self.state {
                            TelnetState::Sub(op) => {
                                let msg = TelnetReceive::Sub((op, self.sub_data.clone()));
                                self.sub_data.clear();
                                self.state = TelnetState::Data;
                                // MCCP3 must be enabled on the encoder immediately after receiving
                                // an IAC SB MCCP3 IAC SE.
                                if op == tc::MCCP3 {
                                    //self.mccp3 = true;
                                }
                                return Ok(Some(msg));
                            },
                            _ => {
                                self.app_data.push(tc::SE);
                            }
                        }
                    }
                }
            } else {
                let byte = src.get_u8();

                match self.state {
                    TelnetState::Data => {
                        if self.line_mode {
                            match byte {
                                tc::CR => {

                                },
                                tc::LF => {
                                    let line = self.app_data.to_vec();
                                    self.app_data.clear();
                                    return Ok(Some(TelnetReceive::Line(line)));
                                },
                                _ => {
                                    self.app_data.push(byte);
                                }
                            }
                        } else {
                            // I really don't wanna be using data mode.. ever.
                            return Ok(Some(TelnetReceive::Data(byte)));
                        }
                    },
                    TelnetState::Sub(op) => {
                        self.sub_data.push(byte);
                    }
                }

            }
        }
    }
}

impl Encoder<TelnetSend> for TelnetCodec {
    type Error = io::Error;

    fn encode(&mut self, item: TelnetSend, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut outgoing = BytesMut::with_capacity(32);

        match item {
            TelnetSend::Data(data) => {
                outgoing.extend(data);
            },
            TelnetSend::Line(mut data) => {
                if !data.ends_with(&[tc::CR, tc::LF]) {
                    data.push(tc::CR);
                    data.push(tc::LF);
                }
                outgoing.extend(data);
            }
            TelnetSend::Prompt(data) => {
                // Not sure what to do about prompts yet.
            },
            TelnetSend::Command((comm, op)) => {
                outgoing.put_u8(tc::IAC);
                outgoing.put_u8(comm);
                outgoing.put_u8(op);
            },
            TelnetSend::Sub((op, data)) => {
                outgoing.put_u8(tc::IAC);
                outgoing.put_u8(tc::SB);
                outgoing.put_u8(op);
                outgoing.extend(data);
                outgoing.reserve(2);
                outgoing.put_u8(tc::IAC);
                outgoing.put_u8(tc::SE);
                // Compression must be enabled immediately after
                // IAC SB MCCP2 IAC SE is sent.
                if op == tc::MCCP2 {
                    //self.mccp2 = true;
                }
            },
            TelnetSend::RawBytes(data) => {
                outgoing.extend(data);
            }
        }
        if self.mccp2 {

        }
        dst.extend(outgoing);
        Ok(())
    }
}

#[derive(Default)]
pub struct TelnetOptionPerspective {
    pub enabled: bool,
    // Negotiating is true if WE have sent a request.
    pub negotiating: bool
}

#[derive(Default)]
pub struct TelnetOptionState {
    pub client: TelnetOptionPerspective,
    pub server: TelnetOptionPerspective,
    pub start_will: bool,
    pub start_do: bool,
}

impl TelnetOptionState {
    pub fn new(start_will: bool, start_do: bool) -> Self {
        TelnetOptionState {
            client: Default::default(),
            server: Default::default(),
            start_will,
            start_do
        }
    }
}

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
    pub screen_reader: bool
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
            screen_reader: false
        }
    }
}

impl TelnetConfiog {
    pub fn capabilities(&self) -> ClientCapabilities {
        ClientCapabilities {
            text: true,
            utf8: self.utf8,
            html: false,
            mxp: self.mxp,
            data: self.gmcp || self.msdp,
            ansi: self.ansi,
            xterm256: self.xterm256,
            width: self.width,
            height: self.height,
            screen_reader: self.screen_reader
        }
    }
}

pub struct TelnetProtocol<T> {
    op_state: HashMap<u8, TelnetOptionState>,
    enabled: bool,
    config: TelnetConfig,
    conn_id: String,
    handshakes_left: u8,
    ttype_handshakes: u8,
    conn: Framed<T, TelnetCodec>,
    ttype_first: Option<Vec<u8>>,
    tx_protocol: Sender<Msg2Protocol>,
    rx_protocol: Receiver<Msg2Protocol>,
    tx_portal: Sender<Msg2Portal>,
    tx_sessmanager: Sender<Msg2SessionManager>,
    tx_session: Option<Sender<Msg2Session>>
}

impl<T> TelnetProtocol<T> where 
    T: AsyncRead + AsyncWrite + Send + 'static
{
    pub fn new(conn_id: String, conn: T, addr: SocketAddr, tls: bool,
               rx_protocol: Receiver<Msg2Protocol>, tx_portal: Sender<Msg2Portal>,
               tx_sessmanager: Sender<Msg2SessionManager>) -> Self {

        let telnet_codec = Framed::new(conn, TelnetCodec::new(true));

        let mut prot = Self {
            conn_id,
            tx_portal,
            op_state: Default::default(),
            enabled: false,
            config: TelnetConfig::default(),
            tx_protocol,
            rx_protocol,
            conn: telnet_codec,
            handshakes_left: 0,
            ttype_handshakes: 0,
            ttype_first: None,
            tx_sessmanager,
            tx_session: None
        };

        // Create Handlers for options...
        // Spread out for easy commenting-out
        // Code, Will-on-start, Do-on-start, handshakes-involved
        for (b, will_start, do_start, counter) in vec![
            (tc::SGA, true, false, 1),
            (tc::NAWS, false, true, 1),
            (tc::TTYPE, false, true, 1),
            (tc::MXP, true, false, 1),
            (tc::MSSP, true, false, 1),
            //(tc::MCCP2, true, false, 1),
            //(tc::MCCP3, true, false, 1),
            (tc::GMCP, true, false, 1),
            (tc::MSDP, true, false, 1),
            (tc::LINEMODE, false, true, 1),
            (tc::TELOPT_EOR, true, false, 1)
        ] {
            let handler = TelnetOptionState::new(will_start, do_start);
            prot.handshakes_left = prot.handshakes_left + counter;
            prot.op_state.insert(b, handler);
        }

        prot.config.tls = tls;
        prot
    }

    pub async fn run(&mut self) {
        let mut raw_bytes: Vec<u8> = Vec::with_capacity(self.op_state.len());

        for (b, handler) in self.op_state.iter_mut() {

            if handler.start_will {
                handler.server.negotiating = true;
                raw_bytes.push(tc::IAC);
                raw_bytes.push(tc::WILL);
                raw_bytes.push(b.clone());
            }
            if handler.start_do {
                handler.client.negotiating = true;
                raw_bytes.push(tc::IAC);
                raw_bytes.push(tc::DO);
                raw_bytes.push(b.clone());
            }
        }
        // Just packing all of this together so it gets sent at once.
        self.conn.send(TelnetSend::RawBytes(raw_bytes)).await;

        loop {
            tokio::select! {
                t_msg = self.conn.next() => {
                    if let Some(msg) = t_msg {
                        match msg {
                            TelnetReceive::Data(b) => {
                                self.receive_data(b).await;
                            },
                            TelnetReceive::Line(bytes) => {
                                self.receive_line(bytes).await;
                            },
                            TelnetReceive::Sub((op, data)) => {
                                self.receive_sub(op, data).await;
                            },
                            TelnetReceive::Will(op) => self.iac_receive(tc::WILL, op).await,
                            TelnetReceive::Wont(op) => self.iac_receive(tc::WONT, op).await,
                            TelnetReceive::Do(op) => self.iac_receive(tc::DO, op).await,
                            TelnetReceive::Dont(op) => self.iac_receive(tc::DONT, op).await,

                        }
                    }
                },
                p_msg = self.rx_protocol.recv() => {
                    if let Some(msg) = p_msg {
                        match msg {
                            Msg2Protocol::Kill => {

                            }
                        }
                    }
                }
            }
            
        }
    }

    async fn send_iac_command(&mut self, command: u8, op: u8) {
        self.conn.send(TelnetSend::Command((command, op))).await;
    }

    async fn send_sub_data(&mut self, op: u8, data: Vec<u8>) {
        self.conn.send(TelnetSend::Sub((op, data))).await;
    }

    async fn iac_receive(&mut self, command: u8, op: u8) {
        // This means we received an IAC will/wont/do/dont...
        if let Some(handler) = self.op_state.get_mut(&op) {
            // We DO have a handler for this option... that means we support it!

            match command {
                tc::WILL => {
                    // The client has sent a WILL. They either want to Locally-Enable op, or are
                    // doing so at our request.
                    if handler.client.negotiating {
                        handler.client.negotiating = false;
                        if !handler.client.enabled {
                            handler.client.enabled = true;
                            self.enable_client(op).await;
                        }
                    } else {
                        handler.client.negotiating = true;
                        self.send_iac_command(tc::DO, op).await;
                    }
                },
                tc::WONT => {
                    // The client has refused an option we wanted to enable. Alternatively, it has
                    // disabled an option that was on.
                    handler.client.negotiating = false;
                    if handler.client.enabled {
                        handler.client.enabled = false;
                        self.disable_client(op).await;
                    }
                },
                tc::DO => {
                    // The client wants the Server to enable Option, or they are acknowledging our
                    // desire to do so.
                    if handler.server.negotiating {
                        if !handler.server.enabled {
                            handler.server.enabled = true;
                            self.enable_server(op).await;
                        }
                    } else {
                        handler.server.negotiating = true;
                        self.send_iac_command(tc::WILL, op).await;
                    }
                },
                tc::DONT => {
                    // The client wants the server to disable Option, or are they are refusing our
                    // desire to do so.
                    handler.server.negotiating = false;
                    if handler.server.enabled {
                        handler.server.enabled = false;
                        self.disable_server(op).await;
                    }
                },
                _ => {
                    // This cannot actually happen.
                }
            }

        } else {
            let mut response: u8 = 0;
            // We do not have a handler for this option, whatever it is... do not support.
            match command {
                tc::WILL => response = tc::DONT,
                tc::DO => response = tc::WONT,
                _ => {
                    // We're not going to respond in any way to a random DONT or WONT...
                }
            }
            if response > 0 {
                self.send_iac_command(response, op).await;
            }
        }
    }

    async fn enable_client(&mut self, op: u8) {
        match op {
            tc::NAWS => self.config.naws = true,
            tc::TTYPE => {
                self.config.ttype = true;
                self.request_ttype().await;
            },
            tc::LINEMODE => self.config.linemode = true,
            _ => {
                // Whatever this option is.. well, whatever.
            }
        }
    }

    async fn disable_client(&mut self, op: u8) {
        match op {
            tc::NAWS => self.config.naws = false,
            tc::TTYPE => self.config.ttype = false,
            tc::LINEMODE => self.config.linemode = false,
            _ => {
                // Whatever this option is.. well, whatever.
            }
        }
    }

    async fn enable_server(&mut self, op: u8) {
        match op {
            tc::SGA => {
                self.config.sga = true;
            },
            // This won't actually happen right now as MCCP2 is disabled.
            tc::MCCP2 => {
                self.config.mccp2 = true;
                self.send_empty_sub(tc::MCCP2).await;
            }
            _ => {

            }
        }
    }

    async fn disable_server(&mut self, op: u8) {

    }

    async fn receive_sub(&mut self, op: u8, data: Vec<u8>) {
        if !self.op_state.contains_key(&op) {
            // Only if we can get a handler, do we want to care about this.
            // All other sub-data is ignored.
            return;
        }
        match op {
            tc::TTYPE => {
                self.receive_ttype(data).await;
            },
            tc::MCCP2 => {
                // This is already enabled by the reader.
            },
            tc::NAWS => {
                self.receive_naws(data).await;
            }
            _ => {

            }
        }
    }

    async fn receive_line(&mut self, data: Vec<u8>) {
        // This is a line that has already been determined to end in CRLF.
        // This can be directly converted to text.
        let data = String::from_utf8(data);
        if let Ok(text) = data {
            // Only if we can decode this text, and only if we have a linked session, do we pass the
            // text on.
            if let Some(mut chan) = &self.tx_session {
                chan.send(Msg2Session::ClientCommand(self.conn_id.clone(), text)).await;
            }
        }
    }

    async fn receive_data(&mut self, data: u8) {
        // I do not want to be receiving single bytes.
    }

    async fn send_empty_sub(&mut self, op: u8) {
        let data: Vec<u8> = Vec::default();
        self.send_sub_data(op, data).await;
    }

    async fn request_ttype(&mut self) {
        let mut data: Vec<u8> = vec![1];
        self.send_sub_data(tc::TTYPE, data);
    }

    async fn receive_ttype(&mut self, data: Vec<u8>) {
        if self.ttype_handshakes > 2 {
            // No reason to listen to TTYPE anymore.
            return;
        }
        if !data.len() > 1 {
            // if there is no data, ignore this.
            return;
        }
        // If the first byte of data is not u8 == 1, this is not valid.
        let (is, info) = data.split_at(1);
        if is[0] != 1 {
            return;
        }
        let mut incoming = String::from("");
        if let Ok(conv) = String::from_utf8(Vec::from(info)) {
            incoming = conv;
        }
        else {
            // We could not parse this to a string. gonna ignore it.
            return;
        }

        if !incoming.len() > 0 {
            // Not sure how we ended up an empty string, but not gonna allow it.
            return;
        }

        incoming = incoming.to_uppercase();

        match self.ttype_handshakes {
            0 => {
                self.ttype_first = Some(data.clone());
                self.receive_ttype_0(incoming).await;
            },
            1 => {
                let t_first = self.ttype_first.clone();
                if let Some(first) = t_first {
                    if first.eq(&data) {
                        // This client does not support advanced ttype. Ignore further
                        // calls to TTYPE and consider this complete.
                        self.ttype_handshakes = 2;
                        self.receive_ttype_basic().await;
                    } else {
                        self.receive_ttype_1(incoming);
                    }
                }
            },
            2 => {
                self.receive_ttype_2(incoming);
            }
            _ => {}
        }
        self.ttype_handshakes = self.ttype_handshakes + 1;
        if self.ttype_handshakes < 2 {
            self.request_ttype().await;
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
        let results: Vec<&str> = data.splitn(1, " ").collect();
        let value = String::from(results[1]);
        let mtts: usize = value.parse().unwrap_or(0);
        if mtts > 0 {
            return;
        }

    }

    async fn receive_ttype_basic(&mut self) {
        // Not sure if anything needs to happen here yet...
    }


    async fn receive_naws(&mut self, data: Vec<u8>) {
        if data.len() != 4 {
            // Naws data must be 4 bytes. We'll ignore this if it's not.
            return;
        }
        let (width, height) = data.split_at(2);
        let width = u16::from_le_bytes(width.try_into().unwrap());
        let height = u16::from_le_bytes(height.try_into().unwrap());
        self.config.width = width;
        self.config.height = height;
    }
}
