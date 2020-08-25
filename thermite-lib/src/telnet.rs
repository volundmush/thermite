use tokio::{
    prelude::*,
    task::JoinHandle,
    sync::mpsc,
    net::{TcpListener, TcpStream}
};

use tokio_util::codec::{Encoder, Decoder, Framed};

use tokio_rustls::{
    TlsAcceptor,
    server::TlsStream
};

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

use thermite_lib::random_alphanum;

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
pub enum TelnetMessage {
    Line(Vec<u8>),
    Data(u8),
    Will(u8),
    Wont(u8),
    Do(u8),
    Dont(u8),
    Sub((u8, Vec<u8>))
}

pub struct TelMsgHolder {
    msg: TelnetMessage
}

pub trait TelnetMsgHolder {
    fn get_msg(&self) -> TelnetMessage;
}

impl TelnetMsgHolder for TelMsgHolder {

    fn get_msg(&self) -> TelnetMessage {
        self.msg.clone()
    }
}

impl TelMsgHolder {
    pub fn new(msg: TelnetMessage) -> Self {
        TelMsgHolder {
            msg
        }
    }
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
    mccp2: bool,
    mccp3: bool
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
            mccp2: false,
            mccp3: false
        }
    }

    fn try_parse_iac(&mut self, bytes: &[u8]) -> (IacSection, usize) {
        if bytes.len() < 2 {
            return (IacSection::Pending, 0);
        };

        if bytes[1] == tc::IAC {
            // Received IAC IAC which is an escape sequence for IAC / 255.
            return (IacSection::IAC, 2);
        }

        match bytes[1] {
            tc::IAC => (IacSection::IAC, 2),
            tc::SE => {
                // This is the only way to ensure the next decode() is decompressed without
                // waiting for the Protocol to acknowledge it.
                match self.state {
                    TelnetState::Sub(op) => {
                        if op == tc::MCCP3 {
                            self.mccp3 = true;
                        }
                    },
                    _ => {}
                }
                return (IacSection::SE, 2);
            },
            tc::WILL | tc::WONT | tc::DO | tc::DONT | tc::SB => {
                if bytes.len() < 3 {
                    // No further IAC sequences are valid without at least 3 bytes so...
                    return (IacSection::Pending, 0);
                }
                return (IacSection::Command((bytes[1], bytes[2])), 3);
            }
            _ => {
                // Still working on this part. Got more commands to enable...
                return (IacSection::Error, 0)
            }
        }
    }
}

impl Decoder for TelnetCodec {
    type Item = TelnetMessage;
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
                            tc::WILL => return Ok(Some(TelnetMessage::Will(op))),
                            tc::WONT => return Ok(Some(TelnetMessage::Wont(op))),
                            tc::DO => return Ok(Some(TelnetMessage::Do(op))),
                            tc::DONT => return Ok(Some(TelnetMessage::Dont(op))),
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
                                let msg = TelnetMessage::Sub((op, self.sub_data.clone()));
                                self.sub_data.clear();
                                self.state = TelnetState::Data;
                                // MCCP3 must be enabled on the encoder immediately after receiving
                                // an IAC SB MCCP3 IAC SE.
                                if op == tc::MCCP3 {
                                    self.mccp3 = true;
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
                                    return Ok(Some(TelnetMessage::Line(line)));
                                },
                                _ => {
                                    self.app_data.push(byte);
                                }
                            }
                        } else {
                            // I really don't wanna be using data mode.. ever.
                            return Ok(Some(TelnetMessage::Data(byte)));
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
                    self.mccp2 = true;
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
    pub tls: bool
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
            tls: false
        }
    }
}

pub enum Msg2Reader {
    Kill
}

pub struct TelnetReader<T>
{
    pub reader: T,
    pub tx_protocol: mpsc::Sender<Msg2Protocol>,
    pub rx_reader: mpsc::Receiver<Msg2Reader>,
}

impl<T> TelnetReader<T> where
    T: Stream<Item = io::Result<TelnetMessage>> + Unpin
    {
    pub fn new(reader: T, tx_protocol: mpsc::Sender<Msg2Protocol>, rx_reader: mpsc::Receiver<Msg2Reader>) -> Self {
        Self {
            reader,
            tx_protocol,
            rx_reader
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                item = self.reader.next() => {
                    if let Some(tel_msg) = item {
                        match tel_msg {
                            Ok(t_msg) => {
                                self.tx_protocol.send(Msg2Protocol::TelnetMsg(t_msg)).await;
                            }
                            Err(e) => {
                                eprintln!("SOMETHING WENT BOGUS! {}", e);
                                // Will deal with this later.
                            }
                        }
                    }
                },
                Some(msg) = self.rx_reader.recv() => {
                    match msg {
                        Msg2Reader::Kill => {
                            break;
                        }
                    }
                }
            }
        }
    }
}

pub enum Msg2Writer {
    Kill,
    Data(TelnetSend)
}

pub struct TelnetWriter<T> {
    pub writer: T,
    pub tx_protocol: mpsc::Sender<Msg2Protocol>,
    pub rx_writer: mpsc::Receiver<Msg2Writer>,
}

impl<T: Sink<TelnetSend> + Unpin> TelnetWriter<T> {

    pub fn new(writer: T, tx_protocol: mpsc::Sender<Msg2Protocol>, rx_writer: mpsc::Receiver<Msg2Writer>) -> Self {
        Self {
            writer,
            tx_protocol,
            rx_writer
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_writer.recv().await {
                match msg {
                    Msg2Writer::Kill => {
                        break;
                    },
                    Msg2Writer::Data(msg) => {
                        self.writer.send(msg).await;
                    }
                }
            }
        }
    }
}

pub enum Msg2Protocol {
    ReaderDisconnected(Option<String>),
    WriterDisconnected(Option<String>),
    ServerDisconnected(Option<String>),
    TelnetMsg(TelnetMessage)
}

pub struct TelnetProtocol {
    pub op_state: HashMap<u8, TelnetOptionState>,
    pub enabled: bool,
    pub config: TelnetConfig,
    pub conn_id: String,
    pub handshakes_left: u8,
    pub ttype_handshakes: u8,
    pub ttype_first: Option<Vec<u8>>,
    pub reader_handle: JoinHandle<()>,
    pub writer_handle: JoinHandle<()>,
    pub tx_protocol: mpsc::Sender<Msg2Protocol>,
    pub rx_protocol: mpsc::Receiver<Msg2Protocol>,
    pub tx_reader: mpsc::Sender<Msg2Reader>,
    pub tx_writer: mpsc::Sender<Msg2Writer>,
    pub tx_server: mpsc::Sender<Msg2Server>
}

impl TelnetProtocol {
    pub fn new(conn_id: String, conn: impl AsyncRead + AsyncWrite + Send + 'static, tx_server: mpsc::Sender<Msg2Server>, tls: bool) -> Self {

        let telnet_codec = Framed::new(conn, TelnetCodec::new(true));

        let (write, read) = telnet_codec.split();

        let (tx_protocol, rx_protocol) = mpsc::channel(50);
        let (tx_reader, rx_reader) = mpsc::channel(50);
        let (tx_writer, rx_writer) = mpsc::channel(50);

        let mut reader = TelnetReader::new(read, tx_protocol.clone(), rx_reader);

        let mut writer = TelnetWriter::new(write, tx_protocol.clone(), rx_writer);

        let read_handle = tokio::spawn(async move {
            reader.run().await;
        });

        let write_handle = tokio::spawn(async move {
            writer.run().await;
        });

        let mut prot = TelnetProtocol {
            conn_id,
            reader_handle: read_handle,
            writer_handle: write_handle,
            tx_server,
            op_state: Default::default(),
            enabled: false,
            config: TelnetConfig::default(),
            tx_protocol,
            rx_protocol,
            tx_reader,
            tx_writer,
            handshakes_left: 0,
            ttype_handshakes: 0,
            ttype_first: None
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
            (tc::MCCP2, true, false, 1),
            (tc::MCCP3, true, false, 1),
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
        self.tx_writer.send(Msg2Writer::Data(TelnetSend::RawBytes(raw_bytes))).await;

        loop {
            if let Some(msg) = self.rx_protocol.recv().await {
                match msg {
                    Msg2Protocol::ServerDisconnected(reason) => {
                        if let Some(display) = reason {
                            // Do something with the reason I guess?
                        };
                        self.tx_reader.send(Msg2Reader::Kill).await;
                        self.tx_writer.send(Msg2Writer::Kill).await;
                        break;
                    },
                    Msg2Protocol::ReaderDisconnected(reason) => {
                        if let Some(display) = reason {
                            // Do something with the reason I guess?
                        };
                        self.tx_writer.send(Msg2Writer::Kill).await;
                        break;
                    },
                    Msg2Protocol::WriterDisconnected(reason) => {
                        if let Some(display) = reason {
                            // Do something with the reason I guess?
                        };
                        self.tx_reader.send(Msg2Reader::Kill).await;
                        break;
                    },
                    Msg2Protocol::TelnetMsg(msg) => {
                        match msg {
                            TelnetMessage::Data(b) => {
                                self.receive_data(b).await;
                            },
                            TelnetMessage::Line(bytes) => {
                                self.receive_line(bytes).await;
                            },
                            TelnetMessage::Sub((op, data)) => {
                                self.receive_sub(op, data).await;
                            },
                            TelnetMessage::Will(op) => self.iac_receive(tc::WILL, op).await,
                            TelnetMessage::Wont(op) => self.iac_receive(tc::WONT, op).await,
                            TelnetMessage::Do(op) => self.iac_receive(tc::DO, op).await,
                            TelnetMessage::Dont(op) => self.iac_receive(tc::DONT, op).await,

                        }
                    }
                }
            }
        }
    }

    async fn send_iac_command(&mut self, command: u8, op: u8) {
        self.tx_writer.send(Msg2Writer::Data(TelnetSend::Command((command, op)))).await;
    }

    async fn send_sub_data(&mut self, op: u8, data: Vec<u8>) {
        self.tx_writer.send(Msg2Writer::Data(TelnetSend::Sub((op, data)))).await;
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
        let text = String::from_utf8_lossy(data.as_slice());
        println!("{}", text);
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

pub enum Msg2Server {
    ClientDisconnected((String, Option<String>)),
    AcceptTcp((TcpStream, SocketAddr)),
    AcceptTls((TlsStream<TcpStream>, SocketAddr))
}


pub struct Connection {
    addr: SocketAddr,
    conn_id: String,
    handle: JoinHandle<()>,
    tx_protocol: mpsc::Sender<Msg2Protocol>,
}

pub enum Msg2Listener {
    Kill
}

pub struct TelnetListener {
    listen_id: String,
    listener: TcpListener,
    tls_acceptor: Option<TlsAcceptor>,
    tx_server: mpsc::Sender<Msg2Server>,
    tx_listener: mpsc::Sender<Msg2Listener>,
    rx_listener: mpsc::Receiver<Msg2Listener>
}

impl TelnetListener {
    pub fn new(listener: TcpListener, tls_acceptor: Option<TlsAcceptor>, listen_id: String, tx_server: mpsc::Sender<Msg2Server>) -> Self {
        let (tx_listener, rx_listener) = mpsc::channel(50);
        Self {
            listen_id,
            tx_server,
            tls_acceptor,
            listener,
            tx_listener,
            rx_listener
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                listen_msg = self.rx_listener.recv() => {
                    if let Some(lis_msg) = listen_msg {
                        match lis_msg {
                            Msg2Listener::Kill => {
                                // I'll worry about this later I guess?
                                break;
                            }
                        }
                    }
                },
                incoming = self.listener.accept() => {
                    match incoming {
                        Ok((tcp_stream, addr)) => {
                            match &self.tls_acceptor {
                                Option::Some(acc) => {
                                    // TLS is engaged. let's get connecting!
                                    let c_acc = acc.clone();
                                    if let Ok(tls_stream) = c_acc.accept(tcp_stream).await {
                                        self.tx_server.send(Msg2Server::AcceptTls((tls_stream, addr))).await;
                                    } else {
                                        // Not sure what to do if TLS fails...
                                    }
                                },
                                Option::None => {
                                    // TLS is not engaged.
                                    self.tx_server.send(Msg2Server::AcceptTcp((tcp_stream, addr))).await;
                                }
                            }
                        },
                        Err(e) => {

                        }
                    }
                },
            }
        }
    }

}

pub struct Listener {
    addr: SocketAddr,
    listen_id: String,
    handle: JoinHandle<()>,
    tx_listener: mpsc::Sender<Msg2Listener>
}

pub struct TelnetServer {
    connections: HashMap<String, Connection>,
    listeners: HashMap<String, Listener>,
    pub tx_server: mpsc::Sender<Msg2Server>,
    rx_server: mpsc::Receiver<Msg2Server>,
}

impl TelnetServer {

    pub fn new() -> Self {
        let (tx_server, rx_server) = mpsc::channel(50);
        Self {
            connections: Default::default(),
            listeners: Default::default(),
            tx_server,
            rx_server,
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                srv_msg = self.rx_server.recv() => {
                    if let Some(sr_msg) = srv_msg {
                        match sr_msg {
                            Msg2Server::ClientDisconnected((id, reason)) => {
                                // I'll worry about this later I guess?
                                self.connections.remove(&id);
                            },
                            Msg2Server::AcceptTcp((stream, addr)) => {
                                self.accept(stream, addr, false);
                            },
                            Msg2Server::AcceptTls((stream, addr)) => {
                                self.accept(stream, addr, true);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn listen(&mut self, listen_id: String, listener: TcpListener, tls: Option<TlsAcceptor>) {
        if self.listeners.contains_key(&listen_id) {
            return;
        }
        let addr = listener.local_addr().unwrap();
        let mut listener = TelnetListener::new(listener, tls, listen_id.clone(), self.tx_server.clone());
        let tx_listener = listener.tx_listener.clone();

        let handle = tokio::spawn(async move {listener.run().await});

        let mut listen_stub = Listener {
            addr,
            listen_id: listen_id.clone(),
            handle,
            tx_listener
        };
        self.listeners.insert(listen_id, listen_stub);
    }

    fn accept(&mut self, conn: impl AsyncRead + AsyncWrite + Send + 'static, addr: SocketAddr, tls: bool) {
        let new_id = self.generate_id();
        let mut protocol = TelnetProtocol::new(new_id.clone(), conn, self.tx_server.clone(), tls);
        let tx_protocol = protocol.tx_protocol.clone();
        let handle = tokio::spawn(async move {protocol.run().await;});
        let conn_data = Connection {
            addr,
            conn_id: new_id.clone(),
            handle,
            tx_protocol
        };
        self.connections.insert(new_id, conn_data);
    }

    fn generate_id(&self) -> String {
        loop {
            let new_id: String = random_alphanum(16);
            if !self.connections.contains_key(&new_id) {
                return new_id;
            }
        }
    }
}
