use tokio_util::codec::{Encoder, Decoder};
use bytes::{BytesMut, Buf, BufMut, Bytes};
use std::io;
use crate::codes;

enum TelnetState {
    Data,
    Sub(u8),
}

#[derive(Clone, Debug)]
pub enum TelnetError {
    BufferReached,
    NAWS,
    TTYPE,
}

#[derive(Clone, Debug)]
pub enum TelnetEvent {
    Negotiate(u8, u8),
    Line(String),
    Prompt(String),
    SubNegotiate(u8, Bytes),
    Data(Bytes),
    // This is width then height
    NAWS(u16, u16),
    TTYPE(String),
    Command(u8),
    Error(TelnetError)
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
    line_mode: bool,
    state: TelnetState,
    app_data: BytesMut,
    sub_data: BytesMut,
}

impl TelnetCodec {
    pub fn new(line_mode: bool, max_buffer: usize) -> Self {
        TelnetCodec {
            line_mode,
            state: TelnetState::Data,
            app_data: BytesMut::with_capacity(max_buffer),
            sub_data: BytesMut::with_capacity(max_buffer)
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
                (IacSection::Command(bytes[1]), 1)
            }
        }
    }
}

impl Decoder for TelnetCodec {
    type Item = TelnetEvent;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {

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
                        if self.app_data.remaining_mut() > 0 {
                            self.app_data.put_u8(codes::IAC);
                        } else {
                            // Breaking the buffer will cause a disconnect.
                            return Err(Self::Error::new(io::ErrorKind::WriteZero, format!("Reached maximum buffer size of: {}", self.app_data.capacity())));
                        }
                    },
                    IacSection::Command(op) => return Ok(Some(TelnetEvent::Command(op))),
                    IacSection::SE => {
                        match self.state {
                            TelnetState::Sub(op) => {
                                // Most sub-negotiation will happen application-side, but we can do
                                // some standard feature supports here.
                                let mut answer = Ok(None);
                                match op {
                                    codes::NAWS => {
                                        // NAWS must be 4 bytes that can be converted to u16s.
                                        // Reject malformed NAWS.
                                        if self.sub_data.len() == 4 {
                                            let width = self.sub_data.get_u16_be();
                                            let height = self.sub_data.get_u16_be();
                                            answer = Ok(Some(TelnetEvent::NAWS(width, height)))
                                        } else {
                                            answer = Err(Self::Error::new(io::ErrorKind::Other, "Received improperly formatted NAWS data."));
                                        }
                                    },
                                    codes::TTYPE => {
                                        // Type data is always a 0 byte followed by an ASCII string.
                                        // Reject anything that doesn't follow this pattern.
                                        if self.sub_data.len() > 2 {
                                            // The first byte is 0, the rest must be a string. If
                                            // we don't have at least two bytes we cannot proceed.
                                            if self.sub_data.get_u8() == 0 {
                                                // If 'is' isn't 0, this is improper and we will not
                                                // bother converting to string.
                                                if let Ok(conv) = String::from_utf8(self.sub_data.clone().to_vec()) {
                                                    answer = Ok(Some(TelnetEvent::TTYPE(conv)));
                                                } else {

                                                }
                                            } else {

                                            }
                                        } else {
                                            
                                        }
                                    },
                                    // This is either unrecognized or app-specific.
                                    _ => answer = Ok(Some(TelnetEvent::SubNegotiate(op, self.sub_data.clone().freeze())))
                                }
                                self.sub_data.clear();
                                self.state = TelnetState::Data;
                                return answer;
                            },
                            _ => {
                                self.app_data.put_u8(codes::SE);
                            }
                        }
                    }
                }
            } else {
                match self.state {
                    TelnetState::Data => {
                        if self.line_mode {
                            match src[0] {
                                codes::CR => {
                                    // Just ignoring CRs for now so I don't have to bother stripping them.
                                    src.advance(1);
                                },
                                codes::LF => {
                                    // Attempt to convert our data to string and send it.
                                    src.advance(1);
                                    let mut answer: Option<TelnetEvent> = None;
                                    if let Ok(conv) = String::from_utf8(self.app_data.to_vec()) {
                                        return Ok(Some(TelnetEvent::Line(conv)));
                                    }
                                    self.app_data.clear();
                                    return Ok(answer);
                                },
                                _ => {
                                    // We need to grab as much data as possible up to a CR, LF, or IAC in this state.
                                    if let Some(ipos) = src.as_ref().iter().position(|b| b == &codes::IAC || b == &codes::CR || b == &codes::LF) {
                                        let mut data = src.split_to(ipos);
                                        if data.len() > 0 {
                                            if self.app_data.remaining_mut() >= data.len() {
                                                self.app_data.put(data);
                                            } else {
                                                return Err(Self::Error::new(io::ErrorKind::WriteZero, 
                                                    format!("Reached maximum buffer size of: {}", self.app_data.capacity())));
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // I really don't wanna be using data mode.. ever. But if someone else does... this must seek up to
                            // an IAC, at which point everything gets copied forward. If there is no IAC, then it just shunts
                            // EVERYTHING forward.
                            if let Some(ipos) = src.as_ref().iter().position(|b| b == &codes::IAC || b == &codes::CR || b == &codes::LF) {
                                let mut data = src.split_to(ipos);
                                return Ok(Some(TelnetEvent::Data(data.freeze())));
                            } else {
                                let mut data = src.split_to(src.len());
                                return Ok(Some(TelnetEvent::Data(data.freeze())));
                            }
                            
                        }
                    },
                    TelnetState::Sub(op) => {
                        // Processing byte in Sub Negotiation mode. In SB mode we are clear to gobble up as many
                        // bytes as we wish, up to an IAC. In fact, until we get an IAC, we can't do anything but
                        // wait for more bytes.
                        if let Some(ipos) = src.as_ref().iter().position(|b| b == &codes::IAC) {
                            // Split off any available up to an IAC and stuff it in the sub data buffer.
                            let mut data = src.split_to(ipos);
                            if data.len() > 0 {
                                if self.sub_data.remaining_mut() >= data.len() {
                                    self.sub_data.put(data);
                                } else {
                                    return Err(Self::Error::new(io::ErrorKind::WriteZero, 
                                        format!("Reached maximum buffer size of: {}", self.app_data.capacity())));
                                }
                            }
                        } else {
                            return Ok(None)
                        }
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