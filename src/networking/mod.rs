use crate::engine;
use std::collections::HashMap;

pub mod telnet;


pub trait GameConnection {
    fn send_bytes(&mut self, data: &[u8], size: usize);
    fn receive_bytes(&mut self, data: &[u8], size: usize);

    fn start(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }

    fn stop(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }
}

pub trait ConnectionHandler {
    fn start(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }

    fn stop(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }
}

pub struct ConnectionManger<'a> {
    pub connections: Vec<Box<dyn GameConnection>>,
    pub handlers: HashMap<u32, Box<dyn ConnectionHandler>>,
    pub engine: &'a engine::ThermiteEngine
}

impl ConnectionManger<'_> {
    fn start(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }

    fn stop(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }
}
