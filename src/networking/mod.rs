extern crate async_trait;
use async_trait::async_trait;
use crate::engine;
use std::collections::HashMap;
use tokio::task::JoinHandle;
use tokio::net;

pub mod telnet;

#[async_trait]
pub trait GameProtocol {
    async fn send_bytes(&mut self, data: &[u8], size: usize);
    async fn receive_bytes(&mut self, data: &[u8], size: usize);

    async fn start(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }
}

pub struct GameConnection {
    pub id: u32,
    pub socket: tokio::net::TcpStream,
    pub protocol: Box<dyn GameProtocol>,
    pub task: tokio::task::JoinHandle<_>
}

#[async_trait]
pub trait ConnectionHandler {

    async fn create_server(&mut self) -> Result<(), std::io::Error> {

    }

    async fn start(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }
}


pub struct ConnectionHolder {
    pub socket: tokio::net::TcpSocket,
    pub task: tokio::task::JoinHandle<_>
}

pub struct GameServer {
    pub name: String,
    pub listener: tokio::net::TcpListener,
    pub task: tokio::task::JoinHandle<_>,
    pub address: String,
    pub port: u32,
    pub handler: Box<dyn ConnectionHandler>,
    pub connections: HashMap<u32, GameConnection>
}


pub struct ConnectionManager {
    pub last_id: u32,
    pub servers: HashMap<String, GameServer>,

}

pub struct ServerDef {
    pub name: String,
    pub address: String,
    pub port: u32
}

impl ConnectionManager {
    async fn start(&mut self) -> Result<(), std::io::Error> {

        let mut listener = net::TcpListener::bind("10.0.0.225:4200").await.unwrap();
        let server = async move {
            let mut incoming = listener.incoming();
            while let Some(socket_res) = incoming.next().await {
                match socket_res {
                    Ok(socket) => {
                        println!("Accepted connection from {:?}", socket.peer_addr());
                        let protocol = TelnetProtocol {};
                        let new_connection = GameConnection {id: 0, socket: socket, protocol: Box::new(protocol)}
                    }
                    Err(err) => {
                        println!("accept error = {:?}", err);
                    }
                }
            }
        };

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }
}
