use tokio_util::codec::{Encoder, Decoder};
use bytes::{BytesMut, Buf, BufMut, Bytes};
use std::{
    io
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
        let mut out = BytesMut::new();

        match src {
            TelnetEvent::Data(data) => {
                out.reserve(data.len());
                out.put(data);
            },
            TelnetEvent::Negotiate(comm, op) => {
                out.reserve(3);
                out.extend(&[codes::IAC, comm, op]);
            },
            TelnetEvent::SubNegotiate(op, data) => {
                out.reserve(5 + data.len());
                out.extend(&[codes::IAC, codes::SB, op]);
                out.extend(data);
                out.extend(&[codes::IAC, codes::SB]);
            },
            TelnetEvent::Command(byte) => {
                out.reserve(2);
                out.extend(&[codes::IAC, byte]);
            }
        }
        out.freeze()
    }
}

pub struct TelnetCodec {
    max_buffer: usize,
}

impl TelnetCodec {
    pub fn new(max_buffer: usize) -> Self {

        TelnetCodec {
            max_buffer,
        }
    }
}

impl Decoder for TelnetCodec {
    type Item = TelnetEvent;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {

        if src.is_empty() {
            return Ok(None);
        }

        if src[0] == codes::IAC {
            if src.len() > 1 {
                match src[1] {
                    codes::IAC => {
                        // This is an escaped IAC. Send it onwards as data.
                        src.advance(2);
                        let mut data = BytesMut::with_capacity(1);
                        data.put_u8(codes::IAC);
                        return Ok(Some(TelnetEvent::Data(data.freeze())));
                    },
                    codes::WILL | codes::WONT | codes::DO | codes::DONT => {
                        if src.len() > 2 {
                            let answer = TelnetEvent::Negotiate(src[1], src[2]);
                            src.advance(3);
                            return Ok(Some(answer));
                        } else {
                            // Not enough bytes for negotiation...yet.
                            return Ok(None)
                        }
                    },
                    codes::SB => {
                        // Since the valid signature is IAC SB <option> <data> IAC SE, and data might be empty, we need at least 5 bytes.
                        if src.len() > 4 {
                            if let Some(ipos) = src.as_ref().windows(2).position(|b| b[0] == codes::IAC && b[1] == codes::SE) {
                                // Split off any available up to an IAC and stuff it in the sub data buffer.
                                let mut data = src.split_to(ipos);
                                src.advance(2);
                                let discard = data.split_to(3);
                                let answer = TelnetEvent::SubNegotiate(discard[2], data.freeze());
                                return Ok(Some(answer))
                            } else {
                                return Ok(None)
                            }
                        } else {
                            // Not enough bytes for sub-negotiation...yet.
                            return Ok(None)
                        }
                    },
                    _ => {
                        // Anything that's not the above is a simple IAC Command.
                        let cmd = src[1];
                        src.advance(2);
                        return Ok(Some(TelnetEvent::Command(cmd)))
                    }
                }
            } else {
                // Need more bytes than a single IAC...
                return Ok(None)
            }
        } else {
            if let Some(ipos) = src.as_ref().iter().position(|b| b == &codes::IAC) {
                // Split off any available up to an IAC and stuff it in the sub data buffer.
                return Ok(Some(TelnetEvent::Data(src.split_to(ipos).freeze())))
            } else {
                return Ok(Some(TelnetEvent::Data(src.split_to(src.len()).freeze())))
            }
        }
    }
}

impl Encoder<TelnetEvent> for TelnetCodec {
    type Error = io::Error;

    fn encode(&mut self, item: TelnetEvent, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let out = Bytes::from(item);
        dst.reserve(out.len());
        dst.put(out.as_ref());
        Ok(())
    }
}