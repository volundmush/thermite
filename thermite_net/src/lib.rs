use tokio::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use uuid::Uuid;
use std::net::SocketAddr;
use tokio_rustls::rustls::{ Certificate, NoClientAuth, PrivateKey, ServerConfig };
use tokio_rustls::rustls::internal::pemfile::{ certs, rsa_private_keys };
use tokio_rustls::TlsAcceptor;

pub enum TxToServer {

}

pub struct ServerLink {
    name: String,
    address: String,
    port: u16,
    tls: bool,
    tx: Sender<TxToServer>,
}

pub enum TxToConnection {

}

pub struct ConnectionLink {
    pub uuid: Uuid,
    pub server: String,
    pub tx: Sender<TxToConnection>
}

pub enum CloseReason {
    ClientDisconnect,
    ServerKick,
    Timeout,
}

pub struct ServerDef {
    pub name: String,
    pub address: String,
    pub port: u16
}

pub enum TxToNetManager {
    CreateServer(ServerDef),
    RegisterConnection(ConnectionLink),
    CloseConnection((Uuid, CloseReason)),
}

pub struct NetworkManager {
    servers: HashMap<String, ServerLink>,
    connections: HashMap<Uuid, ConnectionLink>,
    rx: Receiver<TxToNetManager>,
    pub tx_master: Sender<TxToNetManager>,
    tls_ready: bool,
    running: bool,
    tls_acceptor: Option<TlsAcceptor>
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
            running: false,
            tls_acceptor: Default::default()
        }
    }
}

impl NetworkManager {

    pub async fn setup_tls(&mut self) {
        // Not sure what else this needs yet, but it will set tls_acceptor to have a TlsAcceptor.

    }

    pub async fn create_server(&mut self, def: ServerDef) -> Result<(), &'static str> {
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

                        match listen.accept().await {
                            Ok((_socket, addr)) => {

                                let mut uuid = uuid::Uuid::new_v4();

                                let (mut tx_channel, mut rx_channel) = channel(100);

                                let mut conn = Connection {
                                    uuid: uuid.clone(),
                                    transport: T::default(),
                                    protocol: P::default(),
                                    handler: H::default(),
                                    stream: _socket,
                                    address: addr.clone(),
                                    tx: tx_new.clone(),
                                    rx: rx_channel
                                };


                                tx_new.send(TxToNetManager::RegisterConnection(
                                    ConnectionLink {
                                        uuid,
                                        server: name.clone(),
                                        address: addr,
                                        tx: tx_channel,
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

pub struct Connection<T: Transport, P: Protocol, H: Handler> {
    uuid: Uuid,
    transport: T,
    protocol: P,
    handler: H,
    stream: TcpStream,
    address: SocketAddr,
    tx: Sender<TxToNetManager>,
    rx: Receiver<TxToConnection>
}

impl Connection<T, P, H> {
    async fn run(&mut self) {
        // this will be run inside a Task...
    }

    async fn read_from_stream(&mut self) {
        // this will get all data from stream, sending it to transport.incoming_bytes()
    }
}

pub trait Transport {
    async fn incoming_bytes(&mut self, &mut conn: Connection<T, P, H>){
        // this should take some sort of
    }
}

pub trait Protocol {

}

pub trait Handler {

}

pub struct TCPTransport {

}

impl Transport for TCPTransport {

}

pub struct TLSTransport {

}

impl Transport for TLSTransport {

}

pub struct TelnetProtocol {

}

impl Protocol for TelnetProtocol {

}

pub struct TelnetHandler {

}

impl Handler for TelnetHandler {

}