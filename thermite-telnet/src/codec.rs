use tokio_util::codec::{Encoder, Decoder};
use bytes::{BytesMut, Buf, BufMut};
use std::io;
use crate::codes;

enum TelnetState {
    Data,
    Sub(u8),
}

// Line was definitely a line.
// Data byte by byte application data... bad news for me.
#[derive(Clone, Debug)]
pub enum TelnetEvent {
    Negotiate(u8, u8),
    Line(String),
    Prompt(String),
    SubNegotiate(u8, BytesMut),
    Data(u8),
    // This is width then height
    NAWS(u16, u16),
    TTYPE(String),
    Command(u8),
}

enum IacSection {
    Negotiate(u8, u8),
    IAC,
    Pending,
    Error,
    SE,
    Command(u8)
}

pub struct TelnetCodec {
    sub_data: BytesMut,
    app_data: BytesMut,
    line_mode: bool,
    state: TelnetState
}

impl TelnetCodec {
    pub fn new(line_mode: bool, max_read_buffer: usize) -> Self {
        TelnetCodec {
            app_data: BytesMut::with_capacity(max_read_buffer),
            sub_data: BytesMut::with_capacity(max_read_buffer),
            line_mode,
            state: TelnetState::Data,
        }
    }
}

impl TelnetCodec {
    fn try_parse_iac(&mut self, bytes: impl Buf) -> (IacSection, usize) {
        if bytes.len() < 2 {
            return (IacSection::Pending, 0);
        };

        if bytes[1] == codes::IAC {
            // Received IAC IAC which is an escape sequence for IAC / 255.
            return (IacSection::IAC, 2);
        }

        match bytes[1] {
            codes::IAC => (IacSection::IAC, 2),
            codes::SE => (IacSection::SE, 2),
            codes::WILL | codes::WONT | codes::DO | codes::DONT | codes::SB => {
                if bytes.len() < 3 {
                    // No further IAC sequences are valid without at least 3 bytes so...
                    (IacSection::Pending, 0)
                } else {
                    (IacSection::Negotiate(bytes[1], bytes[2]), 3)
                }
            }
            _ => {
                // Still working on this part. Got more commands to enable...
                (IacSection::Error, 0)
            }
        }
    }
}

impl Decoder for TelnetCodec {
    type Item = TelnetEvent;
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
                    IacSection::Negotiate(comm, op) => {
                        match comm {
                            codes::WILL | codes::WONT | codes::DO | codes::DONT => return Ok(Some(TelnetEvent::Negotiate(comm, op))),
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
                    IacSection::Command(op) => return Ok(Some(TelnetEvent::Command(op))),
                    IacSection::SE => {
                        match self.state {
                            TelnetState::Sub(op) => {
                                // Most sub-negotiation will happen application-side, but we can do
                                // some standard feature supports here.
                                let mut answer: Option<TelnetEvent> = None;
                                match op {
                                    codes::NAWS => {
                                        // NAWS must be 4 bytes that can be converted to u16s.
                                        // Reject malformed NAWS.
                                        if self.sub_data.len() == 4 {
                                            let (width, height) = self.sub_data.split_at(2);
                                            let width = u16::from_be_bytes(width.unwrap());
                                            let height = u16::from_be_bytes(height.unwrap());
                                            answer = Some(TelnetEvent::NAWS(width, height))
                                        }
                                    },
                                    codes::TTYPE => {
                                        // Type data is always a 0 byte followed by an ASCII string.
                                        // Reject anything that doesn't follow this pattern.
                                        if self.sub_data.len() > 2 {
                                            // The first byte is 0, the rest must be a string. If
                                            // we don't have at least two bytes we cannot proceed.
                                            let (is, info) = data.split_at(1);
                                            if is[0] == 0 {
                                                // If 'is' isn't 0, this is improper and we will not
                                                // bother converting to string.
                                                let mut incoming = String::from("");
                                                if let Ok(conv) = String::from_utf8(Vec::from(info)) {
                                                    answer = Some(TelnetEvent::TTYPE(conv));
                                                }
                                            }
                                        }
                                    },
                                    // This is either unrecognized or app-specific.
                                    _ => Some(TelnetEvent::SubNegotiate(op, self.sub_data.clone()))
                                }
                                self.sub_data.clear();
                                self.state = TelnetState::Data;
                                return Ok(answer);
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
                                    // Attempt to convert our data to string and send it.
                                    let mut answer: Option<TelnetEvent> = None;
                                    if let Ok(conv) = String::from_utf8(self.app_data.to_vec()) {
                                        answer = Some(TelnetEvent::Line(conv));
                                    }
                                    self.app_data.clear();
                                    return Ok(answer);
                                },
                                _ => {
                                    self.app_data.push(byte);
                                }
                            }
                        } else {
                            // I really don't wanna be using data mode.. ever.
                            return Ok(Some(TelnetEvent::Data(byte)));
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