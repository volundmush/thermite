use tokio_util::codec::{Encoder, Decoder};
use bytes::{BytesMut, Buf, BufMut};
use std::io;
use crate::codes;

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
}

impl TelnetCodec {
    fn try_parse_iac(&mut self, bytes: &[u8]) -> (IacSection, usize) {
        if bytes.len() < 2 {
            return (IacSection::Pending, 0);
        };

        if bytes[1] == codes::IAC {
            // Received IAC IAC which is an escape sequence for IAC / 255.
            return (IacSection::IAC, 2);
        }

        match bytes[1] {
            codes::IAC => (IacSection::IAC, 2),
            codes::SE => {
                // This is the only way to ensure the next decode() is decompressed without
                // waiting for the Protocol to acknowledge it.
                match self.state {
                    TelnetState::Sub(op) => {
                        if op == codes::MCCP3 {
                            self.mccp3 = true;
                        }
                    },
                    _ => {}
                }
                return (IacSection::SE, 2);
            },
            codes::WILL | codes::WONT | codes::DO | codes::DONT | codes::SB => {
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
    type Item = TelnetReceive;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // should put something here about checking for WAY TOO MUCH BYTES... and kicking if
        // abuse is detected.

        loop {
            if src.is_empty() {
                return Ok(None);
            }

            if src[0] == codes::IAC {
                let (res, consume) = self.try_parse_iac(src.bytes());
                src.advance(consume);

                match res {
                    IacSection::Error => {

                    },
                    IacSection::Command((comm, op)) => {
                        match comm {
                            codes::WILL => return Ok(Some(TelnetReceive::Will(op))),
                            codes::WONT => return Ok(Some(TelnetReceive::Wont(op))),
                            codes::DO => return Ok(Some(TelnetReceive::Do(op))),
                            codes::DONT => return Ok(Some(TelnetReceive::Dont(op))),
                            codes::SB => {
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
                        self.app_data.push(codes::IAC);
                    },
                    IacSection::SE => {
                        match self.state {
                            TelnetState::Sub(op) => {
                                let msg = TelnetReceive::Sub((op, self.sub_data.clone()));
                                self.sub_data.clear();
                                self.state = TelnetState::Data;
                                // MCCP3 must be enabled on the encoder immediately after receiving
                                // an IAC SB MCCP3 IAC SE.
                                if op == codes::MCCP3 {
                                    //self.mccp3 = true;
                                }
                                return Ok(Some(msg));
                            },
                            _ => {
                                self.app_data.push(codes::SE);
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
                                codes::CR => {

                                },
                                codes::LF => {
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
                if !data.ends_with(&[codes::CR, codes::LF]) {
                    data.push(codes::CR);
                    data.push(codes::LF);
                }
                outgoing.extend(data);
            }
            TelnetSend::Prompt(data) => {
                // Not sure what to do about prompts yet.
            },
            TelnetSend::Command((comm, op)) => {
                outgoing.put_u8(codes::IAC);
                outgoing.put_u8(comm);
                outgoing.put_u8(op);
            },
            TelnetSend::Sub((op, data)) => {
                outgoing.put_u8(codes::IAC);
                outgoing.put_u8(codes::SB);
                outgoing.put_u8(op);
                outgoing.extend(data);
                outgoing.reserve(2);
                outgoing.put_u8(codes::IAC);
                outgoing.put_u8(codes::SE);
                // Compression must be enabled immediately after
                // IAC SB MCCP2 IAC SE is sent.
                if op == codes::MCCP2 {
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