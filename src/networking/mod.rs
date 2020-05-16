use std::mem;
use std::vec;
use crate::engine;
use std::collections::HashMap;

pub mod telnet;


pub trait GameConnection {
    fn process_input_bytes(&mut self, bytes: &[u8], length: usize) -> ();
}

pub trait ConnectionHandler {
    fn start(&self) -> Result<(), std::io::Error> {

        Ok(())
    }

    fn stop(&self) -> Result<(), std::io::Error> {

        Ok(())
    }
}

pub struct ConnectionManger<'a> {
    pub connections: Vec<Box<dyn GameConnection>>,
    pub handlers: HashMap<u32, Box<dyn ConnectionHandler>>,
    pub engine: &'a engine::ThermiteEngine
}

impl ConnectionManger {
    fn start(&self) -> Result<(), std::io::Error> {

        Ok(())
    }

    fn stop(&self) -> Result<(), std::io::Error> {

        Ok(())
    }
}
