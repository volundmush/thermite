use std::{
    collections::HashMap,
    net::SocketAddr
};

use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    net::{TcpListener, TcpStream},
    task::JoinHandle
};

use tokio_rustls::{
    TlsAcceptor,
    server::TlsStream
};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;


#[derive(Debug)]
pub enum Msg2MudProtocol {
    Disconnect,
    Line(String),
    Lines(Vec<String>),
    Prompt(String),
    Text(String),
    GMCP(String, JsonValue),
    // When a game requests a Mud Server Status Protocol message,
    ServerStatus(Vec<(String, String)>),
    GetReady
}

#[derive(Debug)]
pub enum ConnectResponse {
    Ok,
    Error(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    Telnet = 0,
    WebSocket = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCapabilities {
    pub protocol: Protocol,
    pub client_name: String,
    pub client_version: String,
    pub utf8: bool,
    pub html: bool,
    pub mxp: bool,
    pub gmcp: bool,
    pub msdp: bool,
    pub mssp: bool,
    pub color: u8,
    pub width: u16,
    pub height: u16,
    pub screen_reader: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolData {
    pub id: String,
    pub addr: SocketAddr,
    pub tls: bool,
    pub capabilities: ProtocolCapabilities,
    pub json_data: JsonValue
}

// This is received by whatever handles connections once they are ready to join the game.
#[derive(Debug, Clone)]
pub struct ProtocolLink {
    pub conn_id: String,
    pub addr: SocketAddr,
    pub tls: bool,
    pub capabilities: ProtocolCapabilities,
    pub tx_protocol: Sender<Msg2MudProtocol>,
    pub json_data: JsonValue
}

impl ProtocolLink {
    pub fn make_data(&self) -> ProtocolData {
        ProtocolData {
            id: self.conn_id.clone(),
            addr: self.addr.clone(),
            tls: self.tls,
            capabilities: self.capabilities.clone(),
            json_data: self.json_data.clone()
        }
    }
}


// Feed one of these to the ListenManager to implement a connection filter.
#[async_trait::async_trait]
pub trait ListenManagerFilter {
    async fn check(&mut self, addr: &SocketAddr) -> bool;
}

pub enum Msg2Listener {
    Kill
}


pub enum Msg2Factory {
    AcceptTCP(TcpStream, SocketAddr),
    AcceptTLS(TlsStream<TcpStream>, SocketAddr),
    Kill
}


pub struct Listener {
    listen_id: String,
    listener: TcpListener,
    factory: String,
    tls_acceptor: Option<TlsAcceptor>,
    rx_listener: Receiver<Msg2Listener>,
    pub tx_listener: Sender<Msg2Listener>,
    tx_listenmanager: Sender<Msg2ListenManager>
}

impl Listener {
    pub fn new(listener: TcpListener, tls_acceptor: Option<TlsAcceptor>, listen_id: String, tx_listenmanager: Sender<Msg2ListenManager>, factory: &String) -> Self {
        let (tx_listener, rx_listener) = channel(50);
        Self {
            listen_id,
            tx_listenmanager,
            factory: factory.clone(),
            tls_acceptor,
            listener,
            tx_listener,
            rx_listener,
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                listen_msg = self.rx_listener.recv() => {
                    if let Some(lis_msg) = listen_msg {
                        match lis_msg {
                            Msg2Listener::Kill => {
                                // I'll worry about this later I guess?
                                break;
                            }
                        }
                    }
                },
                incoming = self.listener.accept() => {
                    match incoming {
                        Ok((tcp_stream, addr)) => {
                            match &self.tls_acceptor {
                                Option::Some(acc) => {
                                    // TLS is engaged. let's get connecting!
                                    let c_acc = acc.clone();
                                    if let Ok(tls_stream) = c_acc.accept(tcp_stream).await {

                                        let _ = self.tx_listenmanager.send(Msg2ListenManager::AcceptTLS(tls_stream, addr, self.factory.clone())).await;
                                    } else {
                                        // Not sure what to do if TLS fails...
                                    }
                                },
                                Option::None => {
                                    // TLS is not engaged.
                                    let _ = self.tx_listenmanager.send(Msg2ListenManager::AcceptTCP(tcp_stream, addr, self.factory.clone())).await;
                                }
                            }
                        },
                        Err(_e) => {

                        }
                    }
                },
            }
        }
    }
}


pub struct FactoryLink {
    pub factory_id: String,
    pub tx_factory: Sender<Msg2Factory>
}

pub struct ListenerLink {
    pub listen_id: String,
    pub factory: String,
    pub tls: bool,
    pub handle: JoinHandle<()>,
    pub tx_listener: Sender<Msg2Listener>,
}

pub enum Msg2ListenManager {
    Kill,
    AcceptTCP(TcpStream, SocketAddr, String),
    AcceptTLS(TlsStream<TcpStream>, SocketAddr, String)
}


pub struct ListenManager {
    listeners: HashMap<String, ListenerLink>,
    factories: HashMap<String, FactoryLink>,
    pub tx_listenmanager: Sender<Msg2ListenManager>,
    rx_listenmanager: Receiver<Msg2ListenManager>,
    //filter: Option<Box<dyn ListenManagerFilter>>
}

impl ListenManager {

    pub fn new() -> Self {

        let (tx_listenmanager, rx_listenmanager) = channel(50);

        Self {
            listeners: Default::default(),
            factories: Default::default(),
            //filter,
            tx_listenmanager,
            rx_listenmanager
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_listenmanager.recv().await {
                match msg {
                    Msg2ListenManager::Kill => {
                        // This should full stop all listeners and clients and tasks then end this tasks.
                        for (_k, v) in self.listeners.iter_mut() {
                            let _ = v.tx_listener.send(Msg2Listener::Kill).await;
                        }
                        for (_k, v) in self.factories.iter_mut() {
                            let _ = v.tx_factory.send(Msg2Factory::Kill).await;
                        }
                        break;
                    },
                    Msg2ListenManager::AcceptTCP(stream, addr, factory) => {
                        if let Some(factory) = self.factories.get_mut(&factory) {
                            let _ = factory.tx_factory.send(Msg2Factory::AcceptTCP(stream, addr)).await;
                        }
                    },
                    Msg2ListenManager::AcceptTLS(stream, addr, factory) => {
                        if let Some(factory) = self.factories.get_mut(&factory) {
                            let _ = factory.tx_factory.send(Msg2Factory::AcceptTLS(stream, addr)).await;
                        }
                    },
                }
            }
        }
    }

    pub fn register_factory(&mut self, factory: FactoryLink) {
        self.factories.insert(factory.factory_id.clone(), factory);
    }

    pub fn listen(&mut self, listen_id: String, listener: TcpListener, tls: Option<TlsAcceptor>, protocol: &String) {
        if self.listeners.contains_key(&listen_id) {
            return;
        }
        if !self.factories.contains_key(protocol) {
            return;
        }

        let tls_bool = tls.is_some();

        let mut listener = Listener::new(listener, tls.clone(), listen_id.clone(), self.tx_listenmanager.clone(), protocol);
        let tx_listener = listener.tx_listener.clone();

        let handle = tokio::spawn(async move {listener.run().await});

        let listen_link = ListenerLink {
            listen_id: listen_id.clone(),
            factory: protocol.clone(),
            handle,
            tx_listener,
            tls: tls_bool
        };
        self.listeners.insert(listen_id, listen_link);
    }
}