use legion::*;
use std::{
    net::{IpAddr, SocketAddr},
    io::{Read, Write, BufRead}
};
use mio::{TcpListener, TcpStream};
use bytes::{Bytes, BytesMut, Buf, BufMut};


#[derive(Debug)]
pub enum Protocol {
    Telnet,
    WebSocket
}

#[derive(Default, Debug)]
pub struct NetComp {}

#[derive(Debug)]
pub struct Listen {
    pub entity: Entity,
    pub protocol: Protocol,
    pub listener: TcpListener,
    pub addr: SocketAddr,
}

#[derive(Debug)]
pub struct Connection {
    pub entity: Entity,
    pub stream: TcpStream,
    pub addr: SocketAddr,
    pub in_buffer: BytesMut,
    pub out_buffer: BytesMut
}

impl Connection {
    pub new(entity: Entity, stream: TcpStream) -> Self {
        Self {
            entity,
            addr: stream.remote_addr().unwrap(),
            stream,
            in_buffer: BytesMut::with_capacity(1024),
            out_buffer: BytesMut::with_capacity(1024)
        }
    }

    pub fn read_all(&mut self) -> std::io::Result {
        loop {
            let res = self.stream.read(&self.in_buffer);
            match res {
                Ok(rd) => {
                    // cool, we read some bytes!
                },
                Err(e) => {
                    match e {
                        ErrorKind::WouldBlock => {
                            return Ok(())
                        },
                        ErrorKind::Interrupted => {
                            continue
                        },
                        _ => {
                            return Err(e)
                        }
                    }
                }
            }
        }
    }

    pub fn flush(&mut self) -> std::io::Result {
        while self.buffer.has_remaining() {
            let res = self.stream.write(&self.out_buffer);
            match res {
                Ok(written) => {
                   self.buffer.consume(written); 
                },
                Err(e) => {
                    match e {
                        ErrorKind::Interrupted => {
                            continue
                        },
                        ErrorKind::WouldBlock => {
                            return Ok(())
                        },
                        _ => return Err(e)
                    }
                }
            }
        }
        Ok(())
    }
}