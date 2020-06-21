use tokio::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use uuid::Uuid;
use std::net::SocketAddr;

pub enum TxToServer {

}

pub struct Server {
    name: String,
    address: String,
    port: u16,
    tls: bool,
    tx: Sender<TxToServer>,
}

pub enum TxToConnection {

}

pub struct Connection {
    uuid: Uuid,
    server: String,
    tx: Sender<TxToConnection>
}

pub enum CloseReason {
    ClientDisconnect,
    ServerKick,
    Timeout,
}

pub struct ServerDef {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub tls: bool
}

pub struct ConnectionDef {
    pub name: String,
    stream: TcpStream,
    addr: SocketAddr
}

pub enum TxToNetManager {
    CreateServer(ServerDef),
    RegisterConnection(ConnectionDef),
    CloseConnection((Uuid, CloseReason)),
}

pub struct NetworkManager {
    servers: HashMap<String, Server>,
    connections: HashMap<Uuid, Connection>,
    rx: Receiver<TxToNetManager>,
    pub tx_master: Sender<TxToNetManager>,
    tls_ready: bool,
    running: bool,
}

impl Default for NetworkManager {
    fn default() -> Self {
        let (mut tx, mut rx) = channel(100);

        Self {
            servers: Default::default(),
            connections: Default::default(),
            rx,
            tx_master: tx,
            tls_ready: false,
            running: false
        }
    }
}

impl NetworkManager {

    async fn create_server(&mut self, def: ServerDef) -> Result<(), &'static str> {
        if self.servers.contains_key(&def.name) {
            return Err("A server already uses that name!");
        }

        if def.tls && !self.tls_ready {
            return Err("TLS is not properly configured!");
        }

        let mut listener = TcpListener::bind(format!("{}:{}", def.address, def.port)).await;
        match listener {
            Ok(mut listen) => {

                let (mut tx, mut rx) = channel(100);

                self.servers.insert(def.name.clone(), Server {
                    name: def.name.clone(),
                    address: def.address.clone(),
                    port,
                    tls: false,
                    tx

                });
                let mut tx_new = self.tx_master.clone();
                tokio::spawn(async move {

                    loop {
                        println!("SERVER STARTING");
                        match listen.accept().await {
                            Ok((_socket, addr)) => {
                                tx_new.send(TxToNetManager::RegisterConnection(
                                    ConnectionDef {
                                        name: name.clone(),
                                        addr,
                                        stream: _socket
                                    })).await;
                            }
                            Err(e) => {
                                eprintln!("error accepting client: {:?}", e);
                            }
                        }
                    }
                });
                Ok(())
            }
            Err(e) => {
                return Err("Could not bind listener!");
            }
        }
    }

    async fn register_connection(&mut self, def: ConnectionDef) {

    }

    // Make sure to call this from inside a tokio::spawn()
    pub async fn run(&mut self) {
        if self.running {
            return;
        }
        self.running = true;
        while self.running {
            while let Some(msg) = self.rx.await {
                match msg {
                    TxToNetManager::CreateServer(def) => {
                        if let Err(e) = self.create_server(def).await {
                            eprintln!("Could not start server: {:?}", e);
                        }
                    }
                    TxToNetManager::RegisterConnection(def) => {
                        if let Err(e) = self.register_Connection(def).await {
                            eprintln!("Could not accept connection: {:?}", e);
                        }
                    }
                    _ => {
                        eprintln!("GOTTA HANDLE THIS ONE!");
                    }
                }
            }
        };
    }
}