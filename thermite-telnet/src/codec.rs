use tokio_util::codec::{Encoder, Decoder};
use bytes::{BytesMut, Buf, BufMut, Bytes};
use std::{
    io,
    io::{Write},
    collections::HashMap,
};
use crate::codes;

use flate2::{
    write::{ZlibDecoder, ZlibEncoder},
    Compression,
};

use serde_json::{Result as JsonResult, Value as JsonValue};

// This may be useful for certain kinds of applications.
pub enum TelnetConnectionType {
    Server,
    Client
}

// Muds are definitely WAY quirkier than normal telnet applications. This enum allows
// the codec to behave differently in certain situations.
pub enum TelnetConnectionMode {
    Normal,
    Mud
}

enum TelnetState {
    Data,
    Sub(u8),
}

// TelnetEvents are the bread and butter of this Codec.
#[derive(Clone, Debug)]
pub enum TelnetEvent {
    // WILL|WONT|DO|DONT <OPTION>
    Negotiate(u8, u8),

    // A line of text that will defintely end in CRLF. When sending, it will be added if not
    // included. When receiving, it will be stripped.
    Line(String),

    // The prompt that's to be sent to the other party.
    Prompt(String),

    // IAC SB <OPTION> <DATA> IAC SE
    SubNegotiate(u8, Bytes),

    // Raw data. This is used instead of Line when the codec is in Normal mode. The
    // application will have to figure out what these mean.
    Data(Bytes),

    // Negotiate About Window Size. this is used instead of SubNegotiate for convenience.
    // This is width then height.
    NAWS(u16, u16),

    // a MTTS / TTYPE sequence after the 'IS'. For MUDs.
    TTYPE(String),

    // An IAC <command> other than those involved in negotiation and sub-options.
    Command(u8),

    // A Generic Mud Communications Protocol message in either direction.
    // This is sent as IAC SB GMCP <string> <json> IAC SE
    GMCP(String, JsonValue),

    // Mud Server Status Protocol stream of values.
    MSSP(HashMap<String, String>),

    // What it says on the tin. This is never received, only 'sent', and serves as a
    // way to reconfigure the TelnetCodec.
    OutgoingCompression(bool),

    // Same as Outgoing.
    IncomingCompression(bool)
}

enum IacSection {
    Negotiate(u8, u8),
    IAC,
    Pending,
    SE,
    Command(u8)
}

pub struct TelnetCodec {
    conn_type: TelnetConnectionType,
    conn_mode: TelnetConnectionMode,
    max_buffer: usize,
    
    // Used by the decoder to know whether it is in data mode or sub mode.
    state: TelnetState,
    
    // in Mud* mode, app data stores bytes until it reaches a CRLF.
    // This is basically forced Linemode.
    app_data: BytesMut,
    // Sub data holds anything dealing with Subnegotiation IAC SB <OP> <DATA> IAC SE
    sub_data: BytesMut,

    // In-data is a temporary storage buffer for incoming bytes which may need to be compressed.
    in_data: BytesMut,

    // Out data is temporary buffer for outgoing bytes that may need to be compressed.
    out_data: BytesMut,

    zipper: ZlibEncoder<Vec<u8>>,
    unzipper: ZlibDecoder<Vec<u8>>,
    out_compress: bool,
    in_compress: bool
}

impl TelnetCodec {
    pub fn new(conn_type: TelnetConnectionType, conn_mode: TelnetConnectionMode, max_buffer: usize) -> Self {

        TelnetCodec {
            conn_type,
            conn_mode,
            state: TelnetState::Data,
            app_data: BytesMut::with_capacity(max_buffer),
            sub_data: BytesMut::with_capacity(max_buffer),
            in_data: BytesMut::with_capacity(max_buffer),
            out_data: BytesMut::with_capacity(max_buffer),
            zipper: ZlibEncoder::new(Vec::new(), Compression::best()),
            unzipper: ZlibDecoder::new(Vec::new()),
            max_buffer,
            in_compress: false,
            out_compress: false
        }
    }
}

impl TelnetCodec {
    fn try_parse_iac(&mut self) -> (IacSection, usize) {
        let bytes = self.in_data.as_ref();
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
    
    fn enable_incoming_compression(&mut self) -> Result<(), io::Error> {
        // Enables incoming zlib compression. If there is anything in the in-buffer, it will be
        // compressed.
        if !self.in_compress {
            self.in_compress = true;
            // Resets the zipper if this is a restarted compression stream.
            self.unzipper.reset(Vec::new())?;
            // If we have any data in the in-buffer, it is definitely compressed.
            if self.in_data.len() > 0 {
                let mut to_decomp = self.in_data.clone();
                self.in_data.clear();
                self.decompress_into(&to_decomp)?
            } else {
            }
        }
        Ok(())
    }

    fn decompress_into(&mut self, src: &BytesMut) -> Result<(), io::Error> {
        match self.unzipper.write_all(src.as_ref()) {
            Ok(()) => {
                //if let Err(e) = self.unzipper.flush() {
                //    return Err(e);
                //}
                let zbuf = self.unzipper.get_mut();
                if self.in_data.remaining_mut() >= zbuf.len() {
                    self.in_data.put(zbuf.as_ref());
                    zbuf.clear();
                    Ok(())
                } else {
                    Err(io::Error::new(io::ErrorKind::InvalidData,
                                                format!("Reached maximum buffer size of: {}", self.in_data.capacity())))
                }
            },
            Err(e) => {
                Err(io::Error::new(io::ErrorKind::InvalidData,
                                            format!("Zlib Decompression Error: {}", e)))
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
            if self.in_compress {
                match self.decompress_into(&src) {
                    Ok(_) => {},
                    Err(e) => return Err(e),
                }
            } else {
                if self.in_data.remaining_mut() >= src.len() {
                    self.in_data.put(src.as_ref());
                } else {
                    return Err(Self::Error::new(io::ErrorKind::InvalidData,
                                                format!("Reached maximum buffer size of: {}", self.max_buffer)));
                }
            }
            // By this point, the src data has been completely consumed into self.in_data
            src.clear();
        }

        // Now that all incoming bytes have been checked for compression, we will loop over it and
        // chop up the bytes into frames. each time this loop encounters an action it can take it
        // will return. So the loop may run once or several times depending on when and how it
        // encounters an IAC (Interpret-as-command). It will likely return with data still in the
        // self.in_data buffer, but the next decode will pick up where it left off.
        // Decode will be called repeatedly even without bytes arriving from the socket, as long
        // as it did not return Ok(None).
        loop {
            if self.in_data.is_empty() {
                return Ok(None);
            }

            if self.in_data[0] == codes::IAC {
                let (res, consume) = self.try_parse_iac();
                self.in_data.advance(consume);

                match res {
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
                        match self.conn_mode {
                            TelnetConnectionMode::Mud => {
                                if self.app_data.remaining_mut() > 0 {
                                    self.app_data.put_u8(codes::IAC);
                                } else {
                                    return Err(Self::Error::new(io::ErrorKind::InvalidData,
                                                                format!("Reached maximum buffer size of: {}", self.app_data.capacity())));
                                }
                            },
                            TelnetConnectionMode::Normal => {
                                // But if we're not in line mode, just send this onwards. It's a bit inefficient but whatever.
                                let mut data = BytesMut::with_capacity(1);
                                data.put_u8(codes::IAC);
                                return Ok(Some(TelnetEvent::Data(data.freeze())));
                            }
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
                                            answer = Err(Self::Error::new(io::ErrorKind::InvalidData,
                                                                          format!("NAWS expects 4 bytes, but received {}", self.sub_data.len())));
                                        }
                                    },
                                    codes::TTYPE => {
                                        // Type data is always a 0 byte followed by an ASCII string.
                                        // Reject anything that doesn't follow this pattern.
                                        if self.sub_data.len() > 2 {
                                            // The first byte is 0, the rest must be a string. If
                                            // we don't have at least two bytes we cannot proceed.
                                            let first = self.sub_data.get_u8();
                                            if first == 0 {
                                                // If 'is' isn't 0, this is improper and we will not
                                                // bother converting to string.
                                                if let Ok(conv) = String::from_utf8(self.sub_data.clone().to_vec()) {
                                                    answer = Ok(Some(TelnetEvent::TTYPE(conv)));
                                                } else {
                                                    answer = Err(Self::Error::new(io::ErrorKind::InvalidData, "MTTS could not convert data to UTF8"));
                                                }
                                            } else {
                                                answer = Err(Self::Error::new(io::ErrorKind::InvalidData,
                                                                              format!("MTTS expects first byte to be 0, got {}", first)));
                                            }
                                        } else {
                                            answer = Err(Self::Error::new(io::ErrorKind::InvalidData, format!("MTTS expects IS data to be longer.")));
                                        }
                                    },
                                    codes::GMCP => {
                                        // We need to de-serialize this into a String command and JSON.
                                        // #TODO: Above.
                                    },
                                    codes::MSSP => {
                                        // We must de-serialize Mud Server Status Protocol into a HashMap<String, String>.
                                        // #TODO: Above.
                                    }
                                    // This is either unrecognized or app-specific. We will pass it up to the application to handle.
                                    _ => answer = Ok(Some(TelnetEvent::SubNegotiate(op, self.sub_data.clone().freeze())))
                                }
                                self.sub_data.clear();
                                self.state = TelnetState::Data;
                                return answer;
                            },
                            _ => {
                                match self.conn_mode {
                                    TelnetConnectionMode::Mud => {
                                        if self.app_data.remaining_mut() > 0 {
                                            self.app_data.put_u8(codes::SE);
                                        } else {
                                            return Err(Self::Error::new(io::ErrorKind::InvalidData,
                                                                        format!("Reached maximum buffer size of: {}", self.max_buffer)));
                                        }
                                    },
                                    TelnetConnectionMode::Normal => {
                                        let mut data = BytesMut::with_capacity(1);
                                        data.put_u8(codes::SE);
                                        return Ok(Some(TelnetEvent::Data(data.freeze())));
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                match self.state {
                    TelnetState::Data => {
                        match self.conn_mode {
                            TelnetConnectionMode::Mud => {
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
                                    answer = Err(Self::Error::new(io::ErrorKind::InvalidData, format!("Reached maximum buffer size of: {}", self.max_buffer)));
                                }
                                let to_advance = cur.len();
                                self.in_data.advance(to_advance);
                                if endline {
                                    if let Ok(conv) = String::from_utf8(self.app_data.to_vec()) {
                                        answer = Ok(Some(TelnetEvent::Line(conv)));
                                    }
                                    // Advancing by two more due to the CRLF.
                                    self.in_data.advance(2);
                                    self.app_data.clear();
                                }
                                return answer;
                            },
                            TelnetConnectionMode::Normal => {
                                // I really don't wanna be using data mode.. ever. But if someone else does... this must seek up to
                                // an IAC, at which point everything gets copied forward. If there is no IAC, then it just shunts
                                // EVERYTHING forward.
                                if let Some(ipos) = self.in_data.as_ref().iter().position(|b| b == &codes::IAC || b == &codes::CR || b == &codes::LF) {
                                    let data = self.in_data.split_to(ipos);
                                    return Ok(Some(TelnetEvent::Data(data.freeze())));
                                } else {
                                    let data = self.in_data.split_to(self.in_data.len());
                                    return Ok(Some(TelnetEvent::Data(data.freeze())));
                                }
                            }
                        }
                    },
                    TelnetState::Sub(op) => {
                        // Processing byte in Sub Negotiation mode. In SB mode we are clear to gobble up as many
                        // bytes as we wish, up to an IAC. In fact, until we get an IAC, we can't do anything but
                        // wait for more bytes.
                        if let Some(ipos) = self.in_data.as_ref().iter().position(|b| b == &codes::IAC) {
                            // Split off any available up to an IAC and stuff it in the sub data buffer.
                            let data = self.in_data.split_to(ipos);
                            if data.len() > 0 {
                                if self.sub_data.remaining_mut() >= data.len() {
                                    self.sub_data.put(data);
                                } else {
                                    return Err(Self::Error::new(io::ErrorKind::InvalidData, 
                                        format!("Reached maximum buffer size of: {}", self.max_buffer)));
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
        println!("Sending TelnetEvent: {:?}", item);
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
            TelnetEvent::SubNegotiate(op, data) => {
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
                self.out_data.put(width.to_be_bytes().as_ref());
                self.out_data.put(height.to_be_bytes().as_ref());
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
            },
            TelnetEvent::GMCP(comm, data) => {
                let outjson = data.to_string();
                self.out_data.reserve(6 + comm.len() + outjson.len());
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SB);
                self.out_data.put_u8(codes::GMCP);
                self.out_data.put(comm.as_bytes());
                self.out_data.put_u8(32);
                self.out_data.put(outjson.as_bytes());
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
                    self.out_data.put(k.as_bytes());
                    self.out_data.put_u8(2);
                    self.out_data.put(v.as_bytes());
                }
                self.out_data.put_u8(codes::IAC);
                self.out_data.put_u8(codes::SE);
            }
            TelnetEvent::IncomingCompression(op) => {
                if op && !self.in_compress {
                    // We are enabling outgoing compression. set toggles and compress any buffered
                    // bytes in self.in_data
                    if let Err(e) = self.enable_incoming_compression() {
                        return Err(e);
                    }
                } else {
                    if self.in_compress {
                        self.in_compress = false;
                    }
                }
            },
            TelnetEvent::OutgoingCompression(op) => {
                if op {
                    if !self.out_compress {
                        self.out_compress = true;
                        if let Err(e) = self.zipper.reset(Vec::new()) {
                            return Err(e);
                        }
                    }
                } else {
                    if self.out_compress {
                        self.out_compress = false;
                    }
                }
            }
        }

        if self.out_compress {
            match self.zipper.write_all(self.out_data.as_ref()) {
                Ok(()) => {
                    if let Ok(_) = self.zipper.flush() {
                        let zbuf = self.zipper.get_mut();
                        dst.reserve(zbuf.len());
                        dst.put(zbuf.as_ref());
                        zbuf.clear();
                    }
                },
                Err(e) => {
                    return Err(Self::Error::new(io::ErrorKind::InvalidData,
                                                format!("Zlib Decompression Error: {}", e)));
                }
            }
        } else {
            dst.reserve(self.out_data.len());
            dst.put(self.out_data.as_ref());
        }
        self.out_data.clear();
        Ok(())
    }
}