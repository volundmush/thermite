use crate::networking::GameConnection;
use std::collections::HashMap;

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

#[derive(Default)]
pub struct TelnetOptionPerspective {
    pub enabled: bool,
    pub negotiating: bool
}

pub struct TelnetConnection {
    option_state_server: HashMap<u8, TelnetOptionPerspective>,
    option_state_client: HashMap<u8, TelnetOptionPerspective>,
    conn_state: TelnetState,
    handshake_count: u8,
    data_buffer: Vec<u8>,
    last_data_byte: u8,
    sub_buffer: Vec<u8>
    
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

    // This is called when we receive an IAC WILL <op> sequence.
    fn process_will(&mut self, op: u8) -> () {

    }

    // This is called when we receive an IAC WONT <op> sequence.
    fn process_wont(&mut self, op: u8) -> () {

    }

    // This is called when we receive an IAC DO <op> sequence.
    fn process_do(&mut self, op: u8) -> () {

    }

    // This is called when we receive an IAC DONT <op> sequence.
    fn process_dont(&mut self, op: u8) -> () {

    }
}

impl GameConnection for TelnetConnection {
    fn process_input_bytes(&mut self, bytes: &[u8], length: usize) -> () {
        
        for n in 0..=length {
            b = bytes[n];
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
                // We are here because we received a CR byte. If the next is an LF, then process
                // it as a command!

                // If we are in this state then we are preparing to enter the InSubnegotiation state.
                TelnetState::Subnegotiation => self.conn_state = TelnetState::InSubnegotiation(b),

                // While in the InSubnegotiation state, we should gather all bytes until we receive
                // an IAC then we switch to SubEscaped.
                TelnetState::InSubnegotiation(op) => {
                    match b {
                        tc::IAC => self.conn_state = TelnetState::SubEscaped(op),
                        _ => self.sub_buffer.append(b)
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
                            self.sub_buffer.append(b);
                            self.conn_state = TelnetState::InSubnegotiation(sub)
                        }
                    }
                }

                TelnetState::InCommand(com) => {
                    match com {
                        tc::WILL => self.process_will(b),
                        tc::WONT => self.process_wont(b),
                        tc::DO => self.process_do(b),
                        tc::DONT => self.process_dont(b),
                        // This last one really can't happen, but gotta be exhaustive.
                        _ => {}
                    }
                    self.conn_state = TelnetState::Data;
                }
            }
        }
    }
}