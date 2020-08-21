use tokio::prelude::*;
use std::collections::HashMap;
use std::vec::Vec;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{Instant};
use bytes::{Bytes, Buf, BytesMut, BufMut};
use crate::net::{MsgManagerToConnection, MsgConnectionToManager, NetworkManager, Connection, Protocol};
use std::io::Write;
use std::net::{SocketAddr};
use tokio::task::JoinHandle;
use tokio::sync::mpsc;
use tokio::net::{TcpStream, TcpListener};
use tokio::net::tcp::{ReadHalf, WriteHalf, OwnedReadHalf, OwnedWriteHalf};
use std::fmt::Write;

pub mod tc {
    pub const NULL: u8 = 0;
    pub const BEL: u8 = 7;
    pub const CR: u8 = 13;
    pub const LF: u8 = 10;
    pub const SGA: u8 = 3;
    pub const NAWS: u8 = 31;
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
    Escaped,
    Subnegotiation,
    InSubnegotiation(u8),
    SubEscaped(u8),
    InCommand(u8),
    NewLine
}

#[derive(Default)]
pub struct TelnetOptionPerspective {
    pub enabled: bool,
    pub negotiating: bool
}

#[derive(Default)]
pub struct TelnetOptionState {
    pub us: TelnetOptionPerspective,
    pub them: TelnetOptionPerspective,
}

pub struct TelnetConfig {
    pub client_name: String,
    pub client_version: String,
    pub encoding: String,
    pub utf8: bool,
    pub ansi: bool,
    pub xterm256: bool,
    pub width: usize,
    pub height: usize,
    pub gmcp: bool,
    pub msdp: bool,
    pub mxp: bool,
    pub mccp2: bool,
    pub mccp3: bool,
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
            mccp3: false,
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

pub enum Msg2TelnetReader {
    Kill,
    Compress(bool)
}

pub struct TelnetReader {
    pub reader: OwnedReadHalf,
    pub tx_protocol: mpsc::Sender<Msg2TelnetProtocol>,
    pub rx_reader: mpsc::Receiver<Msg2TelnetReader>,
    pub compress: bool
}

impl TelnetReader {
    pub async fn run(&mut self) {
        let mut buffer = BytesMut::with_capacity(1024);
        let mut length: usize = 0;

        loop {
            tokio::select! {
                length = self.reader.read(buffer.as_mut()) => {
                    let mut data = buffer.clone();
                    data.resize(length);
                    self.tx_protocol.send(Msg2TelnetProtocol::ReceiveBytes(data.freeze()));
                },
                Some(msg) = self.rx_reader.recv() => {
                    match msg {
                        Msg2TelnetReader::Kill => {
                            break;
                        },
                        Msg2TelnetReader::Compress(state) => {
                            self.compress = state;
                        },
                    }
                }
            }
        }
    }
}

pub enum Msg2TelnetWriter {
    Kill,
    Compress(bool),
    Data(Bytes)
}

pub struct TelnetWriter {
    pub writer: OwnedWriteHalf,
    pub tx_protocol: mpsc::Sender<Msg2TelnetProtocol>,
    pub rx_writer: mpsc::Receiver<Msg2TelnetWriter>,
    pub compress: bool
}

impl TelnetWriter {
    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_writer.recv().await {
                match msg {
                    Msg2TelnetWriter::Kill => {
                        self.writer.poll_shutdown();
                        break;
                    },
                    Msg2TelnetWriter::Compress(state) => {
                        self.compress = state;
                    },
                    Msg2TelnetWriter::Data(payload) => {
                        match self.writer.write_all(payload.as_ref()).await {
                            Ok(_) => {},
                            _ => {
                                // oops something went wrong here!
                                eprintln!("COULD NOT WRITE EVERYTHING?! NOW WHAT?!");
                            }
                        }
                    }
                }
            }
        }
    }
}

pub enum Msg2TelnetProtocol {
    ReaderDisconnected(Some<String>),
    WriterDisconnected(Some<String>),
    ServerDisconnected(Some<String>),
    ReceiveBytes(Bytes),
}

pub struct TelnetProtocol {
    pub addr: SocketAddr,
    pub op_state: HashMap<u8, TelnetOptionState>,
    pub conn_state: TelnetState,
    pub handshake_count: u8,
    pub enabled: bool,
    pub app_buffer: BytesMut,
    pub last_data_byte: u8,
    pub sub_buffer: BytesMut,
    pub config: TelnetConfig,
    pub conn_id: String,
    pub reader_handle: JoinHandle<_>,
    pub writer_handle: JoinHandle<_>,
    pub tx_protocol: mpsc::Sender<Msg2TelnetProtocol>,
    pub rx_protocol: mpsc::Receiver<Msg2TelnetProtocol>,
    pub tx_reader: mpsc::Sender<Msg2Reader>,
    pub tx_writer: mpsc::Sender<Msg2Writer>,
    pub tx_server: mpsc::Sender<Msg2TelnetServer>
}

impl TelnetProtocol {
    pub fn new(conn_id: String, conn: TcpStream, tx_server: mpsc::Sender<Msg2TelnetServer>) -> Self {

        let sock_addr = conn.peer_addr().unwrap();

        let (read, write) = conn.into_split();

        let (tx_protocol, rx_protocol) = mpsc::channel(50);
        let (tx_reader, rx_reader) = mpsc::channel(50);
        let (tx_writer, rx_writer) = mpsc::channel(50);

        let mut reader = TelnetReader {
            reader: read,
            tx_protocol: tx_protocol.clone(),
            rx_reader,
            compress: false,
        };

        let mut writer = TelnetWriter {
            writer: write,
            tx_protocol: tx_protocol.clone(),
            rx_writer,
            compress: false
        };

        let read_handle = tokio::spawn(async move {
            reader.run();
        });

        let write_handle = tokio::spawn(async move {
            writer.run();
        });

        TelnetProtocol {
            conn_id,
            reader_handle: read_handle,
            writer_handle: write_handle,
            addr: sock_addr,
            tx_server,
            conn_state: TelnetState::Data,
            op_state: HashMap::default(),
            handshake_count: 10,
            enabled: false,
            app_buffer: BytesMut::with_capacity(1024),
            last_data_byte: 0,
            sub_buffer: BytesMut::with_capacity(1024),
            config: TelnetConfig::default(),
            tx_protocol,
            rx_protocol,
            tx_reader,
            tx_writer,
        }
    }
}

impl TelnetProtocol {
    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_protocol.recv().await => {
                match msg {
                    Msg2TelnetProtocol::ServerDisconnected(reason) => {
                        if let Some(display) = reason {
                            // Do something with the reason I guess?
                        };
                        self.tx_reader.send(Msg2TelnetReader::Kill);
                        self.tx_writer.send(Msg2TelnetWriter::Kill);
                        break;
                    },
                    Msg2TelnetProtocol::ReaderDisconnected(reason) => {
                        if let Some(display) = reason {
                            // Do something with the reason I guess?
                        };
                        self.tx_writer.send(Msg2TelnetWriter::Kill);
                        break;
                    },
                    Msg2TelnetProtocol::WriterDisconnected(reason) => {
                        if let Some(display) = reason {
                            // Do something with the reason I guess?
                        };
                        self.tx_reader.send(Msg2TelnetReader::Kill);
                        break;
                    },
                    Msg2TelnetProtocol::ReceiveBytes(data) => {
                        self.receive_bytes(data);
                    }
                }
            }
        }
    }

    fn receive_bytes(&mut self, data: impl Buf) {

        for byte in data.bytes() {
            let b = byte.clone();

            match self.conn_state {
                // We are currently in data mode.
                TelnetState::Data => {
                    match b {
                        tc::IAC => self.conn_state = TelnetState::Escaped,
                        tc::CR => self.conn_state = TelnetState::NewLine,
                        _ => { self.app_buffer.put(b)}
                    }
                }

                TelnetState::NewLine => {
                    match b {
                        tc::LF => {
                            self.conn_state = TelnetState::Data;
                            self.process_line(ctx);
                        }
                        _ => {

                        }
                    }
                }

                // We are currently in escaped mode.
                TelnetState::Escaped => {
                    match b {
                        tc::IAC => {
                            // receiving another IAC puts is back in data mode, and escapes the IAC
                            self.conn_state = TelnetState::Data;
                            self.app_buffer.put(b);
                        }
                        tc::SB => {
                            self.conn_state = TelnetState::Subnegotiation;
                            self.sub_buffer.clear();
                        },
                        tc::WILL | tc::WONT | tc::DO | tc::DONT =>
                            self.conn_state = TelnetState::InCommand(b),
                        _ => {
                            // Lol I have no idea.
                            // There's actually more things that can go here -
                            // I just dunno what to do with them yet.
                        }

                    }
                }

                // If we are in this state then we are preparing to enter the InSubnegotiation state.
                TelnetState::Subnegotiation => self.conn_state = TelnetState::InSubnegotiation(b),

                // While in the InSubnegotiation state, we should gather all bytes until we receive
                // an IAC then we switch to SubEscaped.
                TelnetState::InSubnegotiation(op) => {
                    match b {
                        tc::IAC => self.conn_state = TelnetState::SubEscaped(op),
                        _ => self.sub_buffer.extend(&[b])
                    }
                }

                // If we are in the SubEscaped state, we are looking for an SE byte. Anything else
                // will return us to InSubNegotiation.
                TelnetState::SubEscaped(sub) => {
                    match b {
                        tc::SE => {
                            self.conn_state = TelnetState::Data;
                            self.process_sub_buffer(sub, ctx)
                        }
                        _ => {
                            self.sub_buffer.extend(&[b]);
                            self.conn_state = TelnetState::InSubnegotiation(sub)
                        }
                    }
                }

                TelnetState::InCommand(com) => {
                    match com {
                        tc::WILL | tc::WONT | tc::DO | tc::DONT => self.process_iac_command(com, b, ctx),
                        // This last one really can't happen, but gotta be exhaustive.
                        _ => {}
                    }
                    self.conn_state = TelnetState::Data;
                }
            }
        }
    }

}


pub enum Msg2TelnetServer {
    Disconnected((String, Some<String>)),

}


pub struct TelnetListener {

}


pub struct TelnetConnection {
    addr: SocketAddr,
    conn_id: String,
    handle: JoinHandle<_>,
    tx_protocol: mpsc::Sender<Msg2TelnetProtocol>,
}


pub struct TelnetServer {
    connections: HashMap<String, TelnetConnection>,
    tx_server: mspc::Sender<Msg2TelnetServer>,
    rx_server: mspc::Receiver<Msg2TelnetServer>,
    tx_listener: mspc::Sender<Msg2TelnetListener>
}


impl TelnetActor {

    // This is called when we get an IAC DO/DONT/WILL/WONT <option>
    fn process_iac_command(&mut self, command: u8, op: u8, ctx: &mut Context<Self>) {

        // If response code is not 0, it will be sent.
        let mut response_code: u8 = 0;
        let mut start_option = false;
        let mut stop_option = false;

        // This is supported! Let's process negotiation.
        if let Some(mut option_state) = self.negotiation_state.get_mut(&op)
        {
            // These will be set later to true if we want to make this happen.

            if option_state.enabled {
                match command {
                    tc::WONT | tc::DONT => {
                        stop_option = true;
                    }
                    _ => {
                        // Just gonna ignore this.
                    }
                }
            }
            else {

                match command {
                    tc::WILL => {
                        if option_state.sent_start == tc::DO {
                            start_option = true;
                        }
                        else {
                            option_state.received_start = command;
                            response_code = tc::DO;
                            start_option = true;
                        }
                    }
                    tc::DO => {
                        if option_state.sent_start == tc::WILL {
                            start_option = true;
                        }
                        else {
                            option_state.received_start = command;
                            response_code = tc::WILL;
                            start_option = true;
                        }
                    }
                    _ => {
                        // Ignore requests to stop a thing we're not doing.
                    }
                }

            }
        }

        else {
            // This option is not supported. indicate this by sending IAC DONT <OP>
            match command {
                tc::DO => response_code = tc::WONT,
                tc::WILL => response_code = tc::DONT,
                tc::WONT => response_code = tc::DONT,
                tc::DONT => response_code = tc::WONT,
                _ => {
                    // This really shouldn't happen.
                }
            }
        }

        // Send that response code if it was set!
        if response_code > 0 {
            self.send_command(response_code, op, ctx);
        }
        if stop_option {
            self.disable_option(op, ctx);
        }
        if start_option {
            self.enable_option(op, ctx);
        }

    }

    // This is called to handle special logic when enabling an option.
    fn enable_option(&mut self, op: u8, ctx: &mut Context<Self>) -> () {
        match op {
            tc::MCCP2 => {
                // Just because MCCP2 is enabled as an option, doesn't mean compression is enabled.
                // First we need to send IAC SB MCCP2 IAC SE. THEN we enable compression.
                let mut buffer = BytesMut::with_capacity(5);
                buffer.extend(&[tc::IAC, tc::SB, tc::MCCP2, tc::IAC, tc::SE]);
                self.send_bytes(buffer, ctx);
                self.mccp2 = true;
            }
            other => {

            }
        }
    }

    fn disable_option(&mut self, op: u8, ctx: &mut Context<Self>) {
        match op {
            _ => {

            }
        }
    }

    fn process_data_byte(&mut self, byte: u8, ctx: &mut Context<Self>) {
        self.data_buffer.extend(&[byte]);
    }

    fn process_line(&mut self, ctx: &mut Context<Self>) {
        let mut buffer = self.data_buffer.clone();
        self.data_buffer.clear();
        let command = String::from_utf8_lossy(buffer.bytes().clone());
        self.manager.do_send(MsgConnectionToManager::UserCommand(self.uuid, command.to_string()));
    }

    fn process_sub_buffer(&mut self, byte: u8, ctx: &mut Context<Self>) {

    }

    fn send_command(&mut self, command: u8, option: u8, ctx: &mut Context<Self>) {
        let mut buf = BytesMut::with_capacity(3);
        buf.extend(&[tc::IAC, command, option]);
        self.send_bytes(buf, ctx);
    }

    fn send_bytes(&mut self, mut data: impl Buf, ctx: &mut Context<Self>) {

        if self.mccp2 {
            let mut zlib_out = ZlibEncoder::new(Vec::new(), Compression::best());
            zlib_out.write_all(data.bytes());
            let mut buffer = zlib_out.finish().unwrap();
            self.sink.write(Bytes::from(buffer));
        }
        else {
            self.sink.write(data.to_bytes());
        }
    }
}
