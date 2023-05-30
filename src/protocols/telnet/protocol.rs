use std::{
    collections::{HashMap, HashSet},
    vec::Vec,
    net::SocketAddr,
    time::{Duration, Instant},
    sync::Arc
};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc::{Receiver, Sender, channel},
    time
};

use tokio_util::codec::{Framed};

use tokio_stream::wrappers::IntervalStream;

use bytes::{BytesMut, Bytes, BufMut, Buf};

use futures::{
    sink::{SinkExt},
    stream::{StreamExt}
};

use serde_json::Value as JsonValue;

use once_cell::sync::Lazy;
use tokio::task::yield_now;
use tokio::time::timeout;

use crate::{
    protocols::{
        telnet::{
            codec::{TelnetCodec, TelnetEvent},
            codes as tc
        },
        {ProtocolCapabilities, Color, ProtocolLink, MudData}
    },
    msg::{Msg2MudProtocol, Msg2Portal, Msg2PortalFromClient},
    util::ensure_crlf
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

static TELNET_OPTIONS: Lazy<HashMap<u8, TelnetOption>> = Lazy::new( || {
    let mut map: HashMap<u8, TelnetOption> = Default::default();

    map.insert(tc::SGA, TelnetOption {allow_local: true, allow_remote: false, start_remote: false, start_local: true});
    map.insert(tc::NAWS, TelnetOption {allow_local: false, allow_remote: true, start_remote: true, start_local: false});
    map.insert(tc::MTTS, TelnetOption {allow_local: false, allow_remote: true, start_remote: true, start_local: false});
    //map.insert(tc::MXP, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    map.insert(tc::MSSP, TelnetOption {allow_local: true, allow_remote: false, start_remote: false, start_local: true});
    map.insert(tc::MCCP2, TelnetOption {allow_local: true, allow_remote: false, start_remote: false, start_local: true});
    map.insert(tc::MCCP3, TelnetOption {allow_local: true, allow_remote: false, start_remote: false, start_local: true});
    map.insert(tc::GMCP, TelnetOption {allow_local: true, allow_remote: false, start_remote: false, start_local: true});
    map.insert(tc::MSDP, TelnetOption {allow_local: true, allow_remote: false, start_remote: false, start_local: true});
    map.insert(tc::LINEMODE, TelnetOption {allow_local: false, allow_remote: true, start_remote: true, start_local: false});
    map.insert(tc::TELOPT_EOR, TelnetOption {allow_local: true, allow_remote: false, start_remote: false, start_local: true});
    map
});

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

#[derive(Debug)]
pub struct TelnetTimers {
    pub last_interval: Instant,
    pub last_keepalive: Instant,
}

impl Default for TelnetTimers {
    fn default() -> Self {
        Self {
            last_interval: Instant::now(),
            last_keepalive: Instant::now()
        }
    }
}


pub struct TelnetProtocol<T> {
    // This serves as a higher-level actor that abstracts a bunch of the lower-level
    // nitty-gritty so the Session doesn't need to deal with it.
    conn_id: usize,
    op_state: HashMap<u8, TelnetOptionState>,
    addr: SocketAddr,
    config: ProtocolCapabilities,
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
    app_buffer: BytesMut,
    time_created: Instant,
    time_activity: Instant,
    timers: TelnetTimers,
}


impl<T> TelnetProtocol<T> where T: AsyncRead + AsyncWrite + Send + 'static + Unpin + Sync {
    pub fn new(conn_id: usize, conn: Framed<T, TelnetCodec>, addr: SocketAddr, hostnames: Vec<String>, tls: bool, tx_portal: Sender<Msg2Portal>) -> Self {

        let (tx_protocol, rx_protocol) = channel(10);
        // It reaches here! a println!() works.
        let mut out = Self {
            conn_id,
            addr,
            op_state: Default::default(),
            conn,
            config: Default::default(),
            handshakes_left: Default::default(),
            tx_portal,
            tx_protocol,
            rx_protocol,
            ttype_count: 0,
            ttype_last: None,
            active: false,
            sent_link: false,
            running: true,
            app_buffer: BytesMut::with_capacity(1024),
            time_created: Instant::now(),
            time_activity: Instant::now(),
            timers: Default::default(),
        };
        // Stack overflow before reaching this point.
        out.config.tls = tls;
        out.config.host_address = addr.ip().to_string();
        out.config.host_port = addr.port();
        out.config.host_names = hostnames;
        out
    }

    fn make_link(&self) -> ProtocolLink {
        ProtocolLink {
            conn_id: self.conn_id,
            addr: self.addr.clone(),
            capabilities: self.config.clone(),
            tx_protocol: self.tx_protocol.clone()
        }
    }

    async fn handle_conn(&mut self, t_msg: Option<Result<TelnetEvent, std::io::Error>>) {
        if let Some(msg) = t_msg {
            self.time_activity = Instant::now();
            match msg {
                Ok(msg) => {
                    let _ = self.process_telnet_event(msg).await;
                },
                Err(e) => {
                    let _ = self.tx_portal.send(Msg2Portal::ClientDisconnected(self.conn_id, String::from("dunno yet"))).await;
                    self.running = false;
                }
            }
        } else {
            let _ = self.tx_portal.send(Msg2Portal::ClientDisconnected(self.conn_id, String::from("dunno yet"))).await;
            self.running = false;
        }
    }

    pub async fn run(&mut self) {

        // Initialize Telnet Op handlers.
        for (code, tel_op) in TELNET_OPTIONS.iter() {

            let mut state = TelnetOptionState::default();
            if(tel_op.start_local) {
                state.local.negotiating = true;
                let _ = self.conn.send(TelnetEvent::Negotiate(tc::WILL, *code)).await;
                self.handshakes_left.local.insert(*code);
            }
            if(tel_op.start_remote) {
                state.remote.negotiating = true;
                let _ = self.conn.send(TelnetEvent::Negotiate(tc::DO, *code)).await;
                self.handshakes_left.remote.insert(*code);
            }
            self.op_state.insert(*code, state);

        }

        let mut interval_timer = IntervalStream::new(time::interval(Duration::from_millis(100)));

        let mut in_negotiation_phase = true;
        let negotiation_deadline = time::Instant::now() + Duration::from_millis(100);

        // The main loop which operates the protocol during and after negotiation.
        while self.running {
            tokio::select! {
            t_msg = self.conn.next() => self.handle_conn(t_msg).await,
            p_msg = self.rx_protocol.recv() => {
                if let Some(msg) = p_msg {
                    let _ = self.process_protocol_message(msg).await;
                }
            },
            i_msg = interval_timer.next() => {
                if let Some(ins) = i_msg {
                    let _ = self.handle_interval_timer(ins.into_std()).await;
                }
            }
            _ = time::sleep_until(negotiation_deadline), if in_negotiation_phase => {
                in_negotiation_phase = false;
            }
        }

            // Check if negotiations are complete or timed out
            if in_negotiation_phase && self.handshakes_left.is_empty() {
                in_negotiation_phase = false;
            }

            // If negotiations have just completed or timed out, send the ClientConnected message
            if !in_negotiation_phase && !self.sent_link {
                self.tx_portal.send(Msg2Portal::ClientConnected(self.make_link())).await;
                self.sent_link = true;
                self.active = true;
                self.process_app_buffer().await;
            }
        }
    }

    async fn handle_interval_timer(&mut self, ins: Instant) {
        // Check if the connection has been utterly idle at a network level for too long.
        if(self.time_activity.elapsed().as_secs() > (60 * 30)) {
            // handle disconnect here.
        }

        self.timers.last_interval = ins;
    }

    async fn process_telnet_event(&mut self, msg: TelnetEvent) {
        match msg {
            TelnetEvent::SubNegotiate(op, data) => self.receive_sub(op, data).await,
            TelnetEvent::Negotiate(comm, op) => self.receive_negotiate(comm, op).await,
            TelnetEvent::Command(byte) => {
                let _ = self.process_telnet_command(byte).await;
            },
            TelnetEvent::Data(data) => {
                self.app_buffer.put(data);
                if self.sent_link {
                    let _ = self.process_app_buffer().await;
                }
            }
        }
    }

    async fn process_telnet_command(&mut self, byte: u8) {
        match byte {
            tc::NOP => {},
            _ => {}
        }
    }

    async fn process_app_buffer(&mut self) {
        while let Some(ipos) = self.app_buffer.as_ref().iter().position(|b| b == &tc::LF) {
            let cmd = self.app_buffer.split_to(ipos);
            if let Ok(s) = String::from_utf8(cmd.to_vec()) {
                let _ = self.handle_user_command(s.trim().to_string()).await;
            }
            self.app_buffer.advance(1);
        }
    }

    async fn handle_user_command(&mut self, cmd: String) {
        if cmd.len() > 0 {
            if cmd.starts_with("//") {
                let _ = self.handle_protocol_command(cmd);
            } else if self.sent_link {
                // We must format the command as a Msg2PortalFromClient::Data, so we must encapsulate this in a MudData.
                let d = MudData {
                    cmd: String::from("text"),
                    args: vec![JsonValue::String(cmd)],
                    kwargs: Default::default(),
                };
                let _ = self.tx_portal.send(Msg2Portal::FromClient(self.conn_id, Msg2PortalFromClient::Data(vec![d]))).await;
            }
        }
    }

    async fn handle_protocol_command(&mut self, cmd: String) {
        // TODO: Handle protocol commands
    }

    async fn process_protocol_message(&mut self, msg: Msg2MudProtocol) {
        match msg {
            Msg2MudProtocol::Disconnect => {
                self.running = false;
            },
            Msg2MudProtocol::Data(v) => {
                for d in v {
                    let _ = self.process_protocol_message_data(d).await;
                }
            },
        }
    }

    async fn process_protocol_message_data(&mut self, d: MudData) {

        match d.cmd.as_str() {
            "text" => {
                // d.args is a Vec<JsonValue> and ideally each JsonValue is a string.
                // Just send them all as-is. It's up to the game server to handle line splits.
                for jv in d.args {
                    if let JsonValue::String(s) = jv {
                        let _ = self.conn.send(TelnetEvent::Data(Bytes::from(ensure_crlf(&s)))).await;
                    }
                }
            },
            "prompt" => {
                // Prompts are similar to text but end in IAC GA or something to that effect. TODO.
            }
            "mssp" => {
                // This will handle MSSP (Mud Server Status Protocol) data. For this, we need to
                // extract the key-values from d.kwargs (which must be strings), format them as
                // a sequence of <key> <value>, join them with newlines, and send them as a
                // subnegotiation packet IAC SB <MSSP> <lines> IAC SE.
                let mut mssp_data = Vec::new();
                for (k, v) in d.kwargs {
                    mssp_data.push(format!("{} {}", k, v));
                }
                let mssp_data = mssp_data.join("\r\n");
                let _ = self.conn.send(TelnetEvent::SubNegotiate(tc::MSSP, Bytes::from(mssp_data))).await;
            },
            _ => {
                // Anything that isn't text, a prompt, or MSSP, is going to be sent as GMCP.
                // GMCP data is sent via IAC SB <GMCP> <cmd>[ <json>] IAC SE. the json data part is
                // optional and must be separated from the string <cmd> by a space if present.
                // Since our MudData struct only has args and kwargs, the json data will be sent
                // as a list of args and kwargs. So for example, the client might see:
                // IAC SB GMCP room.data [[], {"name": "The Hall of Limbo", "id": 50}] IAC SE
                // In this case cmd is room.data, and args was an empty vec, but kwargs had an object.
                // The empty data structures must always be sent as [] and {} respectively.
                // For our implementation, the client will ALWAYS be receiving the json even if it's
                // empty, for consistency's sake.
                let mut gmcp_data = Vec::new();
                gmcp_data.push(d.cmd);
                let mut gmcp_json = Vec::new();
                // For our process, let's convert args and kwargs to json and send them as a string.

                gmcp_json.push(JsonValue::Array(d.args));
                gmcp_json.push(JsonValue::Object(d.kwargs.into_iter().map(|(k, v)| (k, v)).collect()));
                let json_data = JsonValue::Array(gmcp_json);

                gmcp_data.push(json_data.to_string());

                let gmcp_out = gmcp_data.join(" ");
                let _ = self.conn.send(TelnetEvent::SubNegotiate(tc::GMCP, Bytes::from(gmcp_out))).await;
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
    }

    async fn enable_remote(&mut self, op: u8) {
        match op {
            tc::NAWS => self.config.naws = true,
            tc::MTTS => {
                self.handshakes_left.ttype.insert(0);
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
            tc::MTTS => {
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
            tc::MCCP2 => {
                self.config.mccp2 = true;
                let _ = self.conn.send(TelnetEvent::SubNegotiate(tc::MCCP2, Bytes::new())).await;
            },
            tc::MCCP3 => {
                self.config.mccp3 = true;
            }
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
            tc::MTTS => {
                let _ = self.receive_ttype(data).await;
            },
            tc::GMCP => {
                if let Ok(s) = String::from_utf8(data.to_vec()) {
                    if s.contains(" ") {
                        let mut parts = s.splitn(2, " ");
                        if let Some(cmd) = parts.next() {
                            if let Some(data) = parts.next() {

                                let d = MudData {
                                    cmd: cmd.to_string(),
                                    args: vec![JsonValue::from(data)],
                                    kwargs: HashMap::new()
                                };
                                let m = Msg2PortalFromClient::Data(vec![d]);
                                let _ = self.tx_portal.send(Msg2Portal::FromClient(self.conn_id, m)).await;
                            }
                        }
                    }
                    else {
                        let d = MudData {
                            cmd: s,
                            args: vec![],
                            kwargs: HashMap::new()
                        };
                        let m = Msg2PortalFromClient::Data(vec![d]);
                        let _ = self.tx_portal.send(Msg2Portal::FromClient(self.conn_id, m)).await;
                    }
                }
            },
            _ => {}
        }
    }

    async fn request_ttype(&mut self) {
        let mut data = BytesMut::with_capacity(1);
        data.put_u8(1);
        let _ = self.conn.send(TelnetEvent::SubNegotiate(tc::MTTS, data.freeze())).await;
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
                        }
                    }
                    return;
                }
                _ => {
                    unreachable!("TTYPE count is out of bounds.");
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
            "ATLANTIS" | "CMUD"  | "KILDCLIENT" | "MUDLET" | "MUSHCLIENT"  | "PUTTY" | "BEIP" | "POTATO" | "TINYFUGUE" => {
                self.config.color = Color::Xterm256;
            },
            _ => {
                extra_check = true;
            }
        }
        if extra_check {
            if (self.config.client_name.starts_with("XTERM") || self.config.client_name.ends_with("-256COLOR")) && self.config.color != Color::TrueColor {
                self.config.color = Color::Xterm256;
            }
        }
    }

    async fn receive_ttype_1(&mut self, data: String) {
        if (data.starts_with("XTERM") || data.ends_with("-256COLOR")) && self.config.color != Color::TrueColor  {
            self.config.color = Color::Xterm256;
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
        if (1 & mtts) == 1 && (self.config.color.clone() as i32) < Color::Standard as i32 {
            self.config.color = Color::Standard;
        }

        if (2 & mtts) == 2 {
            self.config.vt100 = true;
        }
        if (4 & mtts) == 4 {
            self.config.utf8 = true;
        }
        if (8 & mtts) == 8 && (self.config.color.clone() as i32) < Color::Xterm256 as i32 {
            self.config.color = Color::Xterm256;
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
        if (256 & mtts) == 256  && (self.config.color.clone() as i32) < Color::TrueColor as i32 {
            self.config.color = Color::TrueColor;
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
            if (self.config.width != old_width) || (self.config.height != old_height) {
                let _ = self.update_capabilities().await;
            }
        }
    }

    async fn update_capabilities(&mut self) {
        if self.sent_link {
            let _ = self.tx_portal.send(Msg2Portal::FromClient(self.conn_id, Msg2PortalFromClient::Capabilities(self.config.clone()))).await;
        }
    }
}