use tokio::{
    prelude::*,
    sync::mpsc::{Receiver, Sender, channel},
    time,
    task
};

use tokio_util::codec::{Framed};


use std::{
    collections::{HashMap, HashSet},
    io,
    vec::Vec,
    net::SocketAddr,
    convert::TryInto,
    time::Duration
};


use bytes::{BytesMut, Bytes, BufMut, Buf};

use futures::{
    sink::{SinkExt},
    stream::{StreamExt}
};

use thermite_net::{Msg2Factory, FactoryLink, Msg2Portal};
use thermite_telnet::{TelnetCodec, codes as tc, TelnetEvent};
use std::sync::Arc;
use crate::session::{Msg2SessionManager, Msg2MudSession, SessionLink};

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

pub struct TelnetSession<T> {
    // This serves as a higher-level actor that abstracts a bunch of the lower-level
    // nitty-gritty so the Session doesn't need to deal with it.
    op_state: HashMap<u8, TelnetOptionState>,
    telnet_options: Arc<HashMap<u8, TelnetOption>>,
    conn_id: String,
    addr: SocketAddr,
    tls: bool,
    handshakes_left: HashSet<u8>,
    conn: Framed<T, TelnetCodec>,
    pub tx_session: Sender<Msg2MudSession>,
    rx_session: Receiver<Msg2MudSession>,
    tx_manager: Sender<Msg2SessionManager>
}


impl<T> TelnetSession<T> where T: AsyncRead + AsyncWrite + Send + 'static + Unpin + std::marker::Sync {
    pub fn new(conn_id: String, conn: Framed<T, TelnetCodec>, addr: SocketAddr, tls: bool,
               tx_manager: Sender<Msg2SessionManager>, telnet_options: Arc<HashMap<u8, TelnetOption>>,
               op_state: HashMap<u8, TelnetOptionState>) -> Self {

        let (tx_session, rx_session) = channel(50);

        let mut handshakes_left: HashSet<u8> = Default::default();
        for (k, v) in op_state.iter() {
            handshakes_left.insert(k.clone());
        }

        Self {
            conn_id,
            addr,
            telnet_options,
            op_state,
            conn,
            handshakes_left,
            tx_session,
            rx_session,
            tx_manager,
            tls
        }
    }

    fn link(&self) -> ConnectionLink {
        ConnectionLink {
            conn_id: self.conn_id.clone(),
            addr: self.addr.clone(),
            tls: self.tls,
            tx_session: self.tx_session.clone()
        }
    }

    pub async fn run(&mut self, opening: Bytes) {
        
        // Just packing all of this together so it gets sent at once.
        self.conn.send(TelnetEvent::Data(raw_bytes)).await;
        let mut send_chan = self.tx_protocol.clone();

        // Ready a message to fire off quickly for in case
        let mut ready_task = tokio::spawn(async move {
            time::delay_for(Duration::from_millis(500)).await;
            send_chan.send(Msg2MudSession::ProtocolReady).await;
            ()
        });

        loop {
            tokio::select! {
                t_msg = self.conn.next() => {
                    if let Some(msg) = t_msg {
                        if let Ok(msg) = msg {
                            match msg {
                                TelnetEvent::Data(data) => self.receive_data(data).await,
                                TelnetEvent::Line(text) => self.receive_line(text).await,
                                TelnetEvent::SubNegotiate(op, data) => self.receive_sub(op, data).await,
                                TelnetEvent::Negotiate(comm, op) => self.receive_negotiate(comm, op).await,
                                TelnetEvent::NAWS(width, height) => self.receive_naws(width, height).await;
                                TelnetEvent::TTYPE(text) => self.receive_ttype(text).await;
                                TelnetEvent::Error(err) => {},
                                TelnetEvent::Command(byte) => {},
                            }
                        }
                    }
                },
                p_msg = self.rx_session.recv() => {
                    if let Some(msg) = p_msg {
                        match msg {
                            Msg2MudSession::Disconnect => {
                                break;
                            },
                            Msg2MudSession::Line(text) => {

                            },
                            Msg2MudSession::Prompt(text) => {

                            },
                            Msg2MudSession::Data => {

                            },
                            Msg2MudSession::MSSP => {

                            },
                            Msg2MudSession::Ready => {

                            }
                        }
                    }
                }
            }
        }
    }

    async fn receive_negotiate(&mut self, command: u8, op: u8) {
        // This means we received an IAC will/wont/do/dont...
        if let Some(state) = self.op_state.get_mut(&op) {
            // We DO have a handler for this option... that means we support it!

            match command {
                tc::WILL => {
                    // The client has sent a WILL. They either want to Locally-Enable op, or are
                    // doing so at our request.
                    if state.remote.negotiating {
                        state.remote.negotiating = false;
                        if !state.remote.enabled {
                            state.remote.enabled = true;
                            let _ = self.enable_remote(op).await;
                        }
                    } else {
                        state.remote.negotiating = true;
                        let _ = self.conn.send(TelnetEvent::Negotiate(tc::DO, op)).await;
                    }
                },
                tc::WONT => {
                    // The client has refused an option we wanted to enable. Alternatively, it has
                    // disabled an option that was on.
                    state.remote.negotiating = false;
                    if state.remote.enabled {
                        state.remote.enabled = false;
                        let _ = self.disable_remote(op).await;
                    }
                },
                tc::DO => {
                    // The client wants the Server to enable Option, or they are acknowledging our
                    // desire to do so.
                    if state.local.negotiating {
                        if !state.local.enabled {
                            state.local.enabled = true;
                            let _ = self.enable_local(op).await;
                        }
                    } else {
                        state.local.negotiating = true;
                        let _ = self.conn.send(TelnetEvent::Negotiate(tc::WILL, op)).await;
                    }
                },
                tc::DONT => {
                    // The client wants the server to disable Option, or are they are refusing our
                    // desire to do so.
                    state.local.negotiating = false;
                    if state.local.enabled {
                        state.local.enabled = false;
                        let _ = self.disable_local(op).await;
                    }
                },
                _ => {
                    // This cannot actually happen.
                }
            }

        } else {
            // We do not have a handler for this option, whatever it is... do not support.
            let response = match command {
                tc::WILL => tc::DONT,
                tc::DO => tc::WONT,
                _ => 0
            };
            if response > 0 {
                let _ = self.conn.send(TelnetEvent::Negotiate(response, op)).await;
            }
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
            tc::NAWS => self.config.naws = false,
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
                // The codec will handle this.
                let _ = self.conn.send(TelnetEvent::SubNegotiate(tc::MCCP2, Bytes::with_capacity(0))).await;
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
            tc::MCCP2 => {
                // This is already enabled by the reader.
            },
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
            let maybe_chan = self.tx_session.clone();
            if let Some(mut chan) = maybe_chan {
                chan.send(Msg2Session::ClientCommand(text)).await;
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
        let data: Vec<u8> = vec![1];
        self.send_sub_data(tc::TTYPE, data).await;
    }

    async fn receive_ttype(&mut self, incoming: String) {
        if !self.handshakes_left_2.contains(&252)
            && !self.handshakes_left_2.contains(&253)
            && !self.handshakes_left_2.contains(&254) {
            return;
        }

        if incoming.len() == 0 {
            // Not sure how we ended up an empty string, but not gonna allow it.
            return;
        }

        incoming = incoming.to_uppercase();

        let run_first = self.handshakes_left_2.contains(&252);

        if run_first {
            self.ttype_first = Some(data.clone());
            self.receive_ttype_0(incoming).await;
            self.process_handshake(252).await;
            self.request_ttype().await;
            return;
        }

        let run_second = self.handshakes_left_2.contains(&253);

        if run_second {
            let t_first = self.ttype_first.clone();
            if let Some(first) = t_first {
                if first.eq(&data) {
                    // This client does not support advanced ttype. Ignore further
                    // calls to TTYPE and consider this complete.
                    self.process_handshake(253).await;
                    self.process_handshake(254).await;
                } else {
                    self.receive_ttype_1(incoming).await;
                    self.process_handshake(253).await;
                    self.request_ttype().await;
                }
            }
            return;
        }

        let run_third = self.handshakes_left_2.contains(&254);
        if run_third {
            self.receive_ttype_2(incoming).await;
            self.process_handshake(254).await;
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
        if (256 & mtts == 256) {
            self.config.truecolor = true;
        }
        if (512 & mtts == 512) {
            self.config.mnes = true;
        }
    }

    async fn receive_ttype_basic(&mut self) {
        // Not sure if anything needs to happen here yet...
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
}

impl TelnetProtocolFactory {
    pub fn new(factory_id: String, options: HashMap<u8, TelnetOption>) -> Self {
        let (tx_factory, rx_factory) = channel(50);

        // Since this only needs to be done once... we'll clone it from here.
        let mut opstates: HashMap<u8, TelnetOptionState> = Default::default();

        let mut raw_bytes = BytesMut::with_capacity(options.len() * 3);

        for (b, handler) in options.iter() {
            let mut state = TelnetOptionState::default();

            if handler.start_will {
                state.local.negotiating = true;
                raw_bytes.put_u8(tc::IAC);
                raw_bytes.put_u8(tc::WILL);
                raw_bytes.put_u8(b.clone());
            }
            if handler.start_do {
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
            opening_bytes: raw_bytes.freeze()
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
                    Msg2Factory::AcceptTLS(conn_id, stream, addr) => {
                        self.accept(conn_id, stream, addr, true);
                    },
                    Msg2Factory::AcceptTCP(conn_id, stream, addr) => {
                        self.accept(conn_id, stream, addr, false);
                    },
                    Msg2Factory::Kill => {
                        break;
                    }
                }
            }
        }
    }

    fn accept<C>(&mut self, conn_id: String, conn: C, addr: SocketAddr, tls: bool) 
        where C: AsyncRead + AsyncWrite + Send + 'static + Unpin + std::marker::Sync
    {
        let telnet_codec = Framed::new(conn, TelnetCodec::new(true));

        
        let (tx_mudtel, rx_mudtel) = channel(50);
        let (tx_mudconn, rx_mudconn) = channel(50);

        let tel_prot = MudTelnetProtocol::new(conn_id.clone(), telnet_codec, tx_mudtel, rx_mudtel, tx_mudconn, self.telnet_options.clone(), self.telnet_statedef.clone());


    }
}