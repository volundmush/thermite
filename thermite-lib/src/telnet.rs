use tokio::{
    prelude::*,
    sync::mpsc::{Receiver, Sender},
    time,
    task
};

use tokio_util::codec::{Encoder, Decoder, Framed};


use std::{
    collections::{HashMap, HashSet},
    io,
    vec::Vec,
    net::SocketAddr,
    convert::TryInto,
    time::Duration
};


use bytes::{BytesMut, Buf, BufMut};

use futures::{
    sink::{SinkExt},
    stream::{StreamExt}
};

use crate::conn::{Msg2Portal, Msg2SessionManager, Msg2Protocol, Msg2Session, ClientInfo, ClientCapabilities, ProtocolType};
use thermite_telnet::{TelnetCodec, codes as tc, TelnetEvent};

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

pub struct TelnetProtocol<T> {
    op_state: HashMap<u8, TelnetOptionState>,
    addr: SocketAddr,
    enabled: bool,
    config: TelnetConfig,
    conn_id: String,
    handshakes_left: HashSet<u8>,
    handshakes_left_2: HashSet<u8>,
    conn: Framed<T, TelnetCodec>,
    started: bool,
    ttype_first: Option<Vec<u8>>,
    tx_protocol: Sender<Msg2Protocol>,
    rx_protocol: Receiver<Msg2Protocol>,
    tx_portal: Sender<Msg2Portal>,
    tx_sessmanager: Sender<Msg2SessionManager>,
    tx_session: Option<Sender<Msg2Session>>
}

impl<T> TelnetProtocol<T> where 
    T: AsyncRead + AsyncWrite + Send + 'static + Unpin + std::marker::Sync
{
    pub fn new(conn_id: String, conn: T, addr: SocketAddr, tls: bool, tx_protocol: Sender<Msg2Protocol>,
               rx_protocol: Receiver<Msg2Protocol>, tx_portal: Sender<Msg2Portal>,
               tx_sessmanager: Sender<Msg2SessionManager>) -> Self {

        let telnet_codec = Framed::new(conn, TelnetCodec::new(true));

        let mut prot = Self {
            conn_id,
            addr,
            tx_portal,
            op_state: Default::default(),
            enabled: false,
            config: TelnetConfig::default(),
            tx_protocol,
            rx_protocol,
            conn: telnet_codec,
            handshakes_left: HashSet::default(),
            handshakes_left_2: HashSet::default(),
            ttype_first: None,
            tx_sessmanager,
            started: false,
            tx_session: None
        };

        // Create Handlers for options...
        // Spread out for easy commenting-out
        // Code, Will-on-start, Do-on-start, handshakes-involved
        for (b, will_start, do_start) in vec![
            (tc::SGA, true, false),
            (tc::NAWS, false, true),
            (tc::TTYPE, false, true),
            //(tc::MXP, true, false),
            (tc::MSSP, true, false),
            //(tc::MCCP2, true, false, 1),
            //(tc::MCCP3, true, false, 1),
            (tc::GMCP, true, false),
            (tc::MSDP, true, false),
            (tc::LINEMODE, false, true),
            (tc::TELOPT_EOR, true, false)
        ] {
            let handler = TelnetOptionState::new(will_start, do_start);
            prot.op_state.insert(b, handler);
            prot.handshakes_left.insert(b);
            prot.handshakes_left_2.insert(b);
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
        let mut send_chan = self.tx_protocol.clone();

        // Ready a message to fire off quickly for in case
        let mut ready_task = tokio::spawn(async move {
            time::delay_for(Duration::from_millis(500)).await;
            send_chan.send(Msg2Protocol::ProtocolReady).await;
            ()
        });

        loop {
            tokio::select! {
                t_msg = self.conn.next() => {
                    if let Some(msg) = t_msg {
                        if let Ok(msg) = msg {
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
                    }
                },
                p_msg = self.rx_protocol.recv() => {
                    if let Some(msg) = p_msg {
                        match msg {
                            Msg2Protocol::Kill => {
                                break;
                            },
                            Msg2Protocol::Disconnect(reason) => {
                                break;
                            },
                            Msg2Protocol::SessionReady(chan) => {
                                self.tx_session = Some(chan);
                            },
                            Msg2Protocol::ProtocolReady => {
                                self.start_session().await;
                            }
                        }
                    }
                }
            }
            
        }
    }

    fn client_info(&self) -> ClientInfo {
        ClientInfo {
            conn_id: self.conn_id.clone(),
            addr: self.addr.clone(),
            tls: self.config.tls,
            protocol: ProtocolType::Telnet,
            tx_protocol: self.tx_protocol.clone()
        }
    }

    async fn start_session(&mut self) {
        // This leaves _2 still ready to handle ttype in case of strange timing.
        if self.started {
            return;
        }
        self.handshakes_left.clear();
        self.started = true;
        self.tx_sessmanager.send(Msg2SessionManager::ClientReady(self.conn_id.clone(),
                                                                 self.client_info(),
                                                                 self.config.capabilities())).await;
    }

    async fn process_handshake(&mut self, op: u8) {
        if self.handshakes_left.len() == 0 {
            return;
        }
        self.handshakes_left.remove(&op);
        self.handshakes_left_2.remove(&op);
        if self.handshakes_left.len() == 0 {
            self.start_session().await;
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
            self.process_handshake(op).await;

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
                // These are fake codes used to represent TTYPE sub-options.
                self.handshakes_left.insert(252);
                self.handshakes_left_2.insert(252);
                self.handshakes_left.insert(253);
                self.handshakes_left_2.insert(253);
                self.handshakes_left.insert(254);
                self.handshakes_left_2.insert(254);
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

    async fn receive_ttype(&mut self, data: Vec<u8>) {
        if !self.handshakes_left_2.contains(&252)
            && !self.handshakes_left_2.contains(&253)
            && !self.handshakes_left_2.contains(&254) {
            return;
        }
        if data.len() < 2 {
            // if there is no data, ignore this.
            return;
        }
        // If the first byte of data is not u8 == 1, this is not valid.
        let (is, info) = data.split_at(1);
        if is[0] != 0 {
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


    async fn naws(&mut self, width: u16, height: u16) {
        self.config.width = width;
        self.config.height = height;
    }
}
