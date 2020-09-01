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
        let mut capacity = 0;
        if line_mode {
            capacity = max_buffer;
        }
        // Setting an override capacity. in line mode, app_data is never used so we don't need to allocate memory.

        TelnetCodec {
            line_mode,
            state: TelnetState::Data,
            app_data: BytesMut::with_capacity(capacity),
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
                        // if we're in line mode then we should append the escaped 255 to app_data.
                        if self.line_mode {
                            if self.app_data.remaining_mut() > 0 {
                                self.app_data.put_u8(codes::IAC);
                            } else {
                                return Err(Self::Error::new(io::ErrorKind::WriteZero, 
                                    format!("Reached maximum buffer size of: {}", self.app_data.capacity())));
                            }
                        } else {
                             // But if we're not in line mode, just send this onwards. It's a bit inefficient but whatever.
                             let mut data = BytesMut::with_capacity(1);
                             data.put_u8(codes::IAC);
                             return Ok(Some(TelnetEvent::Data(data.freeze())));
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
                                            let width = self.sub_data.get_u16();
                                            let height = self.sub_data.get_u16();
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
                                if self.line_mode {
                                    if self.app_data.remaining_mut() > 0 {
                                        self.app_data.put_u8(codes::SE);
                                    } else {
                                        return Err(Self::Error::new(io::ErrorKind::WriteZero, 
                                            format!("Reached maximum buffer size of: {}", self.app_data.capacity())));
                                    }
                                } else {
                                     // But if we're not in line mode, just send this onwards. It's a bit inefficient but whatever.
                                     let mut data = BytesMut::with_capacity(1);
                                     data.put_u8(codes::SE);
                                     return Ok(Some(TelnetEvent::Data(data.freeze())));
                                }
                            }
                        }
                    }
                }
            } else {
                match self.state {
                    TelnetState::Data => {
                        if self.line_mode {
                            // We need to grab as much data as possible up to an IAC.
                            let mut cur = src.as_ref();
                            if let Some(ipos) = src.as_ref().iter().position(|b| b == &codes::IAC) {
                                // there is an IAC.
                                let (data, _) = cur.split_at(ipos);
                                cur = data;
                            }
                            // If there is no IAC, then 'cur' is the entire current data.
                            // The fact that we were called means that cur contains SOMETHING and it's not an IAC.
                            // Here we will search for a CRLF sequence...
                            let mut endline = false;
                            if let Some(ipos) = cur.windows(2).position(|b| b[0] == codes::CR && b[1] == codes::LF) {
                                // We have an endline. once more, set the cur.
                                let (data, _) = cur.split_at(ipos);
                                cur = data;
                                endline = true;
                            }
                            let mut answer = Ok(None);
                            
                            // We must first add this data to app_data due to possible escaped other sources of text, like escaped IACs.
                            if self.app_data.remaining_mut() >= cur.len() {
                                self.app_data.put(cur);
                            } else {
                                answer = Err(Self::Error::new(io::ErrorKind::WriteZero, format!("Reached maximum buffer size of: {}", self.app_data.capacity())));
                            }
                            src.advance(cur.len());
                            if endline {
                                if let Ok(conv) = String::from_utf8(self.app_data.to_vec()) {
                                    answer = Ok(Some(TelnetEvent::Line(conv)));
                                }
                                // Advancing by two more due to the CRLF.
                                src.advance(2);
                                self.app_data.clear();
                            }
                            return answer;
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

impl Encoder<TelnetEvent> for TelnetCodec {
    type Error = io::Error;

    fn encode(&mut self, item: TelnetEvent, dst: &mut BytesMut) -> Result<(), Self::Error> {

        match item {
            TelnetEvent::Data(data) => {
                dst.reserve(data.len());
                dst.put(data);
            },
            TelnetEvent::Line(mut data) => {
                if !data.ends_with("\r\n") {
                    data.push_str("\r\n");
                }
                dst.reserve(data.len());
                dst.put(data.as_bytes());
            },
            TelnetEvent::Prompt(data) => {
                // Not sure what to do about prompts yet.
            },
            TelnetEvent::Negotiate(comm, op) => {
                dst.reserve(3);
                dst.put_u8(codes::IAC);
                dst.put_u8(comm);
                dst.put_u8(op);
            },
            TelnetEvent::SubNegotiate(op, mut data) => {
                dst.reserve(5 + data.len());
                dst.put_u8(codes::IAC);
                dst.put_u8(codes::SB);
                dst.put_u8(op);
                dst.put(data);
                dst.put_u8(codes::IAC);
                dst.put_u8(codes::SE);
            },
            TelnetEvent::NAWS(width, height) => {
                dst.reserve(9);
                dst.put_u8(codes::IAC);
                dst.put_u8(codes::SB);
                dst.extend(&width.to_be_bytes());
                dst.extend(&height.to_be_bytes());
                dst.put_u8(codes::IAC);
                dst.put_u8(codes::SE);
            },
            TelnetEvent::TTYPE(data) => {
                dst.reserve(data.len() + 6);
                dst.put_u8(codes::IAC);
                dst.put_u8(codes::SB);
                dst.put_u8(codes::TTYPE);
                dst.put_u8(0);
                dst.put(data.as_bytes());
                dst.put_u8(codes::IAC);
                dst.put_u8(codes::SE);
            },
            TelnetEvent::Command(byte) => {
                dst.reserve(2);
                dst.put_u8(codes::IAC);
                dst.put_u8(byte);
            }
            TelnetEvent::Error(err) => {

            }
        }
        Ok(())
    }
}