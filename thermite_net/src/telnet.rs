extern crate tokio;
extern crate async_trait;
use async_trait::async_trait;
use tokio::prelude::*;
use futures::stream::StreamExt;
use tokio::task::JoinHandle;
use tokio::net;

use crate::networking::{GameConnection, ConnectionHandler, ConnectionManager};
use std::collections::HashMap;
use std::thread::JoinHandle;

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
    InCommand(u8)
}

pub struct TelnetConnection {
    option_handler: HashMap<u8, bool>,
    conn_state: TelnetState,
    handshake_count: u8,
    data_buffer: Vec<u8>,
    last_data_byte: u8,
    sub_buffer: Vec<u8>,
    mccp2: bool,
    mccp3: bool,
    
}

impl TelnetConnection {
    // This method should fill up the data_buffer and flush it as a command when it receives a
    // CR LF sequence.
    fn process_data_byte(&mut self, byte: u8) -> () {

    }

    // This is called after receiving an IAC SB <command> <data> IAC SE sequence.
    // The <data> is stored in sub_buffer.
    fn process_sub_buffer(&mut self, op: u8) -> () {

    }
    // This is called when we get an IAC DO/DONT/WILL/WONT <option>
    fn process_iac_command(&mut self, command: u8, op: u8) {

        // If response code is not 0, it will be sent.
        let mut response_code: u8 = 0;

        // This is supported! Let's process negotiation.
        if self.option_handler.contains_key(&op) {

            // Retrieve the OptionState that we know is there.
            let option_enabled = *self.option_handler.get_mut(&op).unwrap();

            // These will be set later to true if we want to make this happen.
            let mut start_option = false;
            let mut stop_option = false;

            match command {
                tc::WILL | tc::DO => {
                    // The client has signified it wants to use <option>.
                    if option_enabled {
                        // Why are we getting this again? We shouldn't!
                        // Telnet RFC states to not acknowledge a second positive ack.
                    }
                    else {
                        // We support this. All systems go.
                        response_code = if command==tc::WILL {tc::DO} else {tc::WILL};
                        start_option = true;
                    }
                }
                tc::DONT | tc::WONT => {
                    // The client has signified it wants to reject <option> or disable it.
                    stop_option = true;
                    response_code = if command==tc::DONT {tc::WONT} else {tc::DONT};
                }
                _ => {
                    // This can't really happen.
                }
            }

            // Send that response code if it was set!
            if response_code > 0 {
                self.send_command(response_code, op);
            }

            // No reason to start it if it's already enabled.
            if start_option & !option_enabled {
                self.option_handler.insert(op, true);
                self.enable_option(op);
            }

            // If we're to stop and it's already enabled, then stop it!
            if stop_option & option_enabled {
                self.option_handler.insert(op, false);
                self.disable_option(op);
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

            // Send that response code if it was set!
            if response_code > 0 {
                self.send_command(response_code, op);
            }
        }

    }

    // This is called to handle special logic when enabling an option.
    fn enable_option(&mut self, op: u8) -> () {
        match op {
            tc::MCCP2 => {
                // Just because MCCP2 is enabled as an option, doesn't mean compression is enabled.
                // First we need to send IAC SB MCCP2 IAC SE. THEN we enable compression.
                let to_send: [u8; 5] = [tc::IAC, tc::SB, tc::MCCP2, tc::IAC, tc::SE];
                self.send_bytes(&to_send, 5);
                self.mccp2 = true;
            }
            _ => {

            }
        }
    }

    fn disable_option(&mut self, op: u8) {
        match op {
            _ => {

            }
        }
    }

    fn send_command(&mut self, command: u8, option: u8) {
        let to_send: [u8; 3] = [tc::IAC, command, option];
        self.send_bytes(&to_send, 3);
    }

    fn process_bytes(&mut self, bytes: &[u8], length: usize) -> () {

        for n in 0..=length {
            let b = bytes[n];
            match self.conn_state {
                // We are currently in data mode.
                TelnetState::Data => {
                    match b {
                        tc::IAC => self.conn_state = TelnetState::Escaped,
                        _ => self.process_data_byte(b)
                    }
                }

                // We are currently in escaped mode.
                TelnetState::Escaped => {
                    match b {
                        tc::IAC => {
                            self.conn_state = TelnetState::Data;
                            self.process_data_byte(b);
                        }
                        tc::SB => self.conn_state = TelnetState::Subnegotiation,
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
                        _ => self.sub_buffer.push(b)
                    }
                }

                // If we are in the SubEscaped state, we are looking for an SE byte. Anything else
                // will return us to InSubNegotiation.
                TelnetState::SubEscaped(sub) => {
                    match b {
                        tc::SE => {
                            self.conn_state = TelnetState::Data;
                            self.process_sub_buffer(sub)
                        }
                        _ => {
                            self.sub_buffer.push(b);
                            self.conn_state = TelnetState::InSubnegotiation(sub)
                        }
                    }
                }

                TelnetState::InCommand(com) => {
                    match com {
                        tc::WILL | tc::WONT | tc::DO | tc::DONT => self.process_iac_command(com, b),
                        // This last one really can't happen, but gotta be exhaustive.
                        _ => {}
                    }
                    self.conn_state = TelnetState::Data;
                }
            }
        }
    }
}

#[async_trait]
impl GameConnection for TelnetConnection {
    async fn send_bytes(&mut self, data: &[u8], size: usize) {
        if self.mccp2 {
            // If mccp2 is on, we need to be sending a zlib compress9 stream.
        }
        else {

        }
    }

    async fn receive_bytes(&mut self, data: &[u8], size: usize) {
        if self.mccp3 {
            // if mccp3 is on, we are RECEIVING a zlib compress9 stream
        }
        else {

        }

        // Now that it's been decompressed if necessary, pass to processing!
        self.process_bytes(data, size);
    }

    async fn start(&mut self) -> Result<(), std::io::Error> {
        let supported_options = vec![tc::SGA, tc::NAWS, tc::MXP, tc::MSSP, tc::MCCP2,
                                     tc::MCCP3, tc::GMCP, tc::MSDP, tc::TTYPE];
        for op in supported_options {
            self.option_handler.insert(op, false);
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }
}

pub struct TelnetConnectionHandler<'a> {
    pub manager: &'a ConnectionManager<'a>,
    pub address: String,
    pub port: u32,
    pub listener: Option<net::TcpListener>,
    pub task: Option<tokio::task::JoinHandle<_>>
}

#[async_trait]
impl ConnectionHandler for TelnetConnectionHandler<'_> {
    async fn start(&mut self) -> Result<(), std::io::Error> {


        Ok(())
    }

    async fn stop(&mut self) -> Result<(), std::io::Error> {
        match self.task {
            Some(handler) => {
                handler.await.expect("Server stopped abnormally!");
            }
            None => {
                println!("No server running!");
            }
        }
        Ok(())
    }
}