use tokio_util::codec::{Encoder, Decoder};
use bytes::{
    BytesMut, Buf, BufMut, Bytes,
    buf::Writer,
};

use std::{
    io
};
use std::io::{Cursor, Read, Write};
use flate2::{
    write::{ZlibEncoder},
    write::{ZlibDecoder},
    Compression
};


use super::codes;

use serde_json::{Result as JsonResult, Value as JsonValue};



// TelnetEvents are the bread and butter of this Codec.
#[derive(Clone, Debug)]
pub enum TelnetEvent {
    // WILL|WONT|DO|DONT <OPTION>
    Negotiate(u8, u8),

    // IAC SB <OPTION> <DATA> IAC SE
    SubNegotiate(u8, Bytes),

    // Raw data. The application will have to figure out what these mean.
    Data(Bytes),

    // An IAC <command> other than those involved in negotiation and sub-options.
    Command(u8)
}

impl From<TelnetEvent> for Bytes {
    fn from(src: TelnetEvent) -> Self {
        match src {
            TelnetEvent::Data(data) => data,
            TelnetEvent::Negotiate(comm, op) => {
                let mut out = BytesMut::with_capacity(3);
                out.extend(&[codes::IAC, comm, op]);
                out.freeze()
            },
            TelnetEvent::SubNegotiate(op, data) => {
                let mut out = BytesMut::with_capacity(5 + data.len());
                out.extend(&[codes::IAC, codes::SB, op]);
                out.extend(data);
                out.extend(&[codes::IAC, codes::SE]);
                out.freeze()
            },
            TelnetEvent::Command(byte) => {
                let mut out = BytesMut::with_capacity(2);
                out.extend(&[codes::IAC, byte]);
                out.freeze()
            }
        }
    }
}

impl TelnetEvent {
    pub fn parse(src: &mut BytesMut) -> Option<Self> {
        if src.is_empty() {
            return None;
        }

        if src[0] == codes::IAC {
            if src.len() > 1 {
                match src[1] {
                    codes::IAC => {
                        // This is an escaped IAC. Send it onwards as data.
                        src.advance(2);
                        let mut data = BytesMut::with_capacity(1);
                        data.put_u8(codes::IAC);
                        Some(TelnetEvent::Data(data.freeze()))
                    },
                    codes::WILL | codes::WONT | codes::DO | codes::DONT => {
                        if src.len() > 2 {
                            let answer = TelnetEvent::Negotiate(src[1], src[2]);
                            src.advance(3);
                            Some(answer)
                        } else {
                            // Not enough bytes for negotiation...yet.
                            None
                        }
                    },
                    codes::SB => {
                        // Since the valid signature is IAC SB <option> <data> IAC SE, and data might be empty, we need at least 5 bytes.
                        if src.len() > 4 {
                            let mut escape_next = false;
                            if let Some(ipos) = src.as_ref().windows(2).enumerate().find_map(|(i, b)| {
                                if escape_next {
                                    escape_next = false;
                                    return None;
                                }

                                if b[0] == codes::IAC {
                                    if b[1] == codes::SE {
                                        return Some(i);
                                    } else if b[1] == codes::IAC {
                                        escape_next = true;
                                    }
                                }
                                None
                            }) {
                                // Split off any available up to an IAC and stuff it in the sub data buffer.
                                let mut data = src.split_to(ipos);
                                src.advance(2);
                                let discard = data.split_to(3);
                                let answer = TelnetEvent::SubNegotiate(discard[2], data.freeze());
                                Some(answer)
                            } else {
                                None
                            }
                        } else {
                            // Not enough bytes for sub-negotiation...yet.
                            None
                        }
                    },
                    _ => {
                        // Anything that's not the above is a simple IAC Command.
                        let cmd = src[1];
                        src.advance(2);
                        Some(TelnetEvent::Command(cmd))
                    }
                }
            } else {
                // Need more bytes than a single IAC...
                None
            }
        } else {
            if let Some(ipos) = src.as_ref().iter().position(|b| b == &codes::IAC) {
                // Split off any available up to an IAC and stuff it in the sub data buffer.
                Some(TelnetEvent::Data(src.split_to(ipos).freeze()))
            } else {
                Some(TelnetEvent::Data(src.split_to(src.len()).freeze()))
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct TelnetCodec {
    max_buffer: usize,
    decoder: Option<ZlibDecoder<Writer<BytesMut>>>,
    encoder: Option<ZlibEncoder<Writer<BytesMut>>>,
}

impl TelnetCodec {
    pub fn new(max_buffer: usize) -> Self {

        TelnetCodec {
            max_buffer,
            decoder: None,
            encoder: None
        }
    }
}

impl Encoder<TelnetEvent> for TelnetCodec {
    type Error = io::Error;

    fn encode(&mut self, item: TelnetEvent, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let start_encoding = self.encoder.is_none() &&
            matches!(item, TelnetEvent::SubNegotiate(codes::MCCP2, _));;

        let b = Bytes::from(item);

        if let Some(encoder) = &mut self.encoder {
            encoder.write_all(b.as_ref());
            encoder.flush()?;
            &dst.writer().write_all(encoder.get_mut().get_mut())?;
            encoder.get_mut().get_mut().clear();
        } else {
            dst.reserve(b.len());
            dst.put(b.as_ref());
        };

        if start_encoding {
            self.encoder = Some(ZlibEncoder::new(BytesMut::new().writer(),
                                                 Compression::best()));
        }

        Ok(())
    }
}

impl Decoder for TelnetCodec {
    type Item = TelnetEvent;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {

        if src.len() > self.max_buffer {
            return Err(Self::Error::from(io::ErrorKind::OutOfMemory));
        }

        let result = if let Some(decoder) = &mut self.decoder {
            decoder.write_all(src)?;
            src.clear();
            decoder.flush()?;
            TelnetEvent::parse(&mut decoder.get_mut().get_mut())
        } else {
            TelnetEvent::parse(src)
        };

        if self.decoder.is_none() {
            match result {
                Some(TelnetEvent::SubNegotiate(codes::MCCP3, _)) => {
                    self.decoder = Some(ZlibDecoder::new(BytesMut::new().writer()));
                },
                _ => {}
            }
        }

        Ok(result)
    }
}
