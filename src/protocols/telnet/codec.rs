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

#[derive(Clone, Debug, Default)]
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

impl Encoder<TelnetEvent> for TelnetCodec {
    type Error = io::Error;

    fn encode(&mut self, item: TelnetEvent, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let out = Bytes::from(item);
        dst.reserve(out.len());
        dst.put(out.as_ref());
        Ok(())
    }
}

impl Decoder for TelnetCodec {
    type Item = TelnetEvent;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        if src.len() > self.max_buffer {
            return Err(Self::Error::from(io::ErrorKind::OutOfMemory));
        }

        let mut position = 0;

        while position < src.len() {
            if src[position] == codes::IAC {
                if position + 1 < src.len() {
                    match src[position + 1] {
                        codes::IAC => {
                            src.advance(position);
                            src.advance(1); // Remove the first IAC
                            src[position] = codes::IAC; // Replace the second IAC with a single IAC
                            position += 1; // Move past the replaced IAC
                        }
                        codes::WILL | codes::WONT | codes::DO | codes::DONT => {
                            if position + 2 < src.len() {
                                let answer = TelnetEvent::Negotiate(src[position + 1], src[position + 2]);
                                src.advance(position);
                                src.advance(3); // Remove the IAC command
                                return Ok(Some(answer));
                            } else {
                                return Ok(None);
                            }
                        }
                        codes::SB => {
                            if position + 4 < src.len() {
                                if let Some(ipos) = src.as_ref()[position + 2..].windows(2).position(|b| b[0] == codes::IAC && b[1] == codes::SE) {
                                    let mut data = src.split_to(position);
                                    src.advance(2);
                                    let discard = data.split_to(3);
                                    let answer = TelnetEvent::SubNegotiate(discard[2], data.freeze());
                                    return Ok(Some(answer));
                                } else {
                                    return Ok(None);
                                }
                            } else {
                                return Ok(None);
                            }
                        }
                        _ => {
                            let cmd = src[position + 1];
                            src.advance(position);
                            src.advance(2); // Remove the IAC command
                            return Ok(Some(TelnetEvent::Command(cmd)));
                        }
                    }
                } else {
                    return Ok(None);
                }
            } else {
                position += 1;
            }
        }

        Ok(Some(TelnetEvent::Data(src.split_to(position).freeze())))
    }
}

