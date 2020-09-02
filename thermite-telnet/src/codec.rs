use tokio_util::codec::{Encoder, Decoder};
use bytes::{BytesMut, Buf, BufMut, Bytes};
use std::{
    io,
    io::{Read, Write},
    collections::HashMap,
};
use crate::codes;

use flate2::{
    write::{ZlibDecoder, ZlibEncoder},
    Compression,
};

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
    Error(TelnetError),
    MCCP2(bool),
    MCCP3(bool),
    Compress2(bool),
    Compress3(bool),
    GMCP(String, String),
    MSSP(HashMap<String, String>)
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
    max_buffer: usize,
    state: TelnetState,
    app_data: BytesMut,
    sub_data: BytesMut,
    in_data: BytesMut,
    out_data: BytesMut,
    zipper: ZlibEncoder<Vec<u8>>,
    unzipper: ZlibDecoder<Vec<u8>>,
    mccp2_enabled: bool,
    mccp2_compress: bool,
    mccp3_enabled: bool,
    mccp3_compress: bool,
    mnes_buffer: HashMap<String, String>
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
            sub_data: BytesMut::with_capacity(max_buffer),
            in_data: BytesMut::with_capacity(max_buffer),
            out_data: BytesMut::with_capacity(max_buffer),
            zipper: ZlibEncoder::new(Vec::new(), Compression::best()),
            unzipper: ZlibDecoder::new(Vec::new()),
            max_buffer,
            mccp2_enabled: false,
            mccp2_compress: false,
            mccp3_enabled: false,
            mccp3_compress: false,
            mnes_buffer: Default::default()
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

        // src has the bytes from the stream. but we want to optionally decompress it into in_data
        if src.len() > 0 {
            if self.mccp3_compress {
                match self.unzipper.write_all(src.as_ref()) {
                    Ok(()) => {
                        self.unzipper.flush();
                        let zbuf = self.unzipper.get_mut();
                        if self.in_data.remaining() >= zbuf.len() {
                            self.in_data.extend(zbuf.as_ref());
                            zbuf.clear();
                        } else {
                            // ERROR!
                        }
                    },
                    Err(e) => {
                        //
                    }
                }
            } else {
                if self.in_data.remaining() >= src.len() {
                    self.in_data.extend(src.as_ref());
                } else {
                    // ERROR!
                }
            }
            // By this point, the src data has been completely consumed into self.in_data
            src.clear();
        }
        
        // #TODO: replace all src refs below with self.in_data

        loop {
            if self.in_data.is_empty() {
                return Ok(None);
            }

            if self.in_data[0] == codes::IAC {
                let (res, consume) = self.try_parse_iac(self.in_data.bytes());
                self.in_data.advance(consume);

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
                                    codes::MCCP3 => {
                                        // Upon receiving IAC SB MCCP3 IAC SE, we enable mccp3_compress.
                                        if !self.mccp3_compress {
                                            self.mccp3_compress = true;
                                            // If we have any data in the in-buffer, it is definitely compressed.
                                            if self.in_data.len() > 0 {
                                                match self.zipper.write_all(self.in_data.as_ref()) {
                                                    Ok(_) => {
                                                        let zbuf = self.zipper.get_mut();
                                                        self.in_data.clear();
                                                        if self.in_data.remaining() >= zbuf.len() {
                                                            self.in_data.extend(zbuf.as_ref());
                                                            zbuf.clear();
                                                        } else {
                                                            // ERROR!
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        answer = Ok(Some(TelnetEvent::Compress3(true)));
                                    }
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
                            let mut cur = self.in_data.as_ref();
                            if let Some(ipos) = self.in_data.as_ref().iter().position(|b| b == &codes::IAC) {
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
                            self.in_data.advance(cur.len());
                            if endline {
                                if let Ok(conv) = String::from_utf8(self.app_data.to_vec()) {
                                    answer = Ok(Some(TelnetEvent::Line(conv)));
                                }
                                // Advancing by two more due to the CRLF.
                                self.in_data.advance(2);
                                self.app_data.clear();
                            }
                            return answer;
                            } else {
                            // I really don't wanna be using data mode.. ever. But if someone else does... this must seek up to
                            // an IAC, at which point everything gets copied forward. If there is no IAC, then it just shunts
                            // EVERYTHING forward.
                            if let Some(ipos) = self.in_data.as_ref().iter().position(|b| b == &codes::IAC || b == &codes::CR || b == &codes::LF) {
                                let mut data = self.in_data.split_to(ipos);
                                return Ok(Some(TelnetEvent::Data(data.freeze())));
                            } else {
                                let mut data = self.in_data.split_to(self.in_data.len());
                                return Ok(Some(TelnetEvent::Data(data.freeze())));
                            }
                            
                        }
                    },
                    TelnetState::Sub(op) => {
                        // Processing byte in Sub Negotiation mode. In SB mode we are clear to gobble up as many
                        // bytes as we wish, up to an IAC. In fact, until we get an IAC, we can't do anything but
                        // wait for more bytes.
                        if let Some(ipos) = self.in_data.as_ref().iter().position(|b| b == &codes::IAC) {
                            // Split off any available up to an IAC and stuff it in the sub data buffer.
                            let mut data = self.in_data.split_to(ipos);
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
                self.out_data.reserve(data.len());
                self.out_data.put(data);
            },
            TelnetEvent::Line(mut data) => {
                if !data.ends_with("\r\n") {
                    data.push_str("\r\n");
                }
                self.out_data.reserve(data.len());
                self.out_data.put(data.as_bytes());
            },
            TelnetEvent::Prompt(data) => {
                // Not sure what to do about prompts yet.
            },
            TelnetEvent::Negotiate(comm, op) => {
                self.out_data.reserve(3);
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(comm);
                self.out_data.put_u8(op);
            },
            TelnetEvent::SubNegotiate(op, mut data) => {
                self.out_data.reserve(5 + data.len());
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SB);
                self.out_data.put_u8(op);
                self.out_data.put(data);
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SE);
            },
            TelnetEvent::NAWS(width, height) => {
                self.out_data.reserve(9);
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SB);
                self.out_data.extend(&width.to_be_bytes());
                self.out_data.extend(&height.to_be_bytes());
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SE);
            },
            TelnetEvent::TTYPE(data) => {
                self.out_data.reserve(data.len() + 6);
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SB);
                self.out_data.put_u8(codes::TTYPE);
                self.out_data.put_u8(0);
                self.out_data.put(data.as_bytes());
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SE);
            },
            TelnetEvent::Command(byte) => {
                self.out_data.reserve(2);
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(byte);
            }
            TelnetEvent::Error(err) => {

            },
            TelnetEvent::GMCP(comm, data) => {
                self.out_data.reserve(6 + comm.len() + data.len());
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SB);
                self.out_data.put_u8(codes::GMCP);
                self.out_data.extend(comm.as_bytes());
                self.out_data.put_u8(32);
                self.out_data.extend(data.as_bytes());
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SE);
            },
            TelnetEvent::MSSP(vals) => {
                self.out_data.reserve(5);
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SB);
                self.out_data.put_u8(codes::MSSP);
                for (k, v) in vals.iter() {
                    self.out_data.reserve(k.len() + v.len() + 2);
                    self.out_data.put_u8(1);
                    self.out_data.extend(k.as_bytes());
                    self.out_data.put_u8(2);
                    self.out_data.extend(v.as_bytes());
                }
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SE);
            }
            TelnetEvent::MCCP2(val) => self.mccp2_enabled = val,
            TelnetEvent::MCCP3(val) => self.mccp3_enabled = val,
            TelnetEvent::Compress2(val) => self.mccp2_compress = val,
            TelnetEvent::Compress3(val) => self.mccp3_compress = val,
        }

        if self.mccp2_compress {
            match self.zipper.write_all(self.out_data.as_ref()) {
                Ok(()) => {
                    if let Ok(_) = self.zipper.flush() {
                        let zbuf = self.zipper.get_mut();
                        dst.reserve(zbuf.len());
                        dst.extend(zbuf.as_ref());
                        zbuf.clear();
                    } 
                },
                Err(e) => {

                }
            }
        } else {
            dst.reserve(self.out_data.len());
            dst.extend(self.out_data.as_ref());
        }
        self.out_data.clear();
        Ok(())
    }
}