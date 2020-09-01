use std::{
    collections::HashMap,
    net::SocketAddr
};

use tokio::{
    prelude::*,
    sync::mpsc::{channel, Receiver, Sender},
    net::{TcpListener, TcpStream},
    task::JoinHandle
};

use tokio_rustls::{
    TlsAcceptor,
    server::TlsStream
};

use thermite_util::text::{
    random_alphanum
};
use std::collections::HashSet;


pub enum Msg2Connection {
    Kill,
    Disconnect(Option<String>),
    ProtocolReady,
    SessionReady(Sender<Msg2Session>)
}


pub enum Msg2Listener {
    Kill
}


pub struct Listener {
    listen_id: String,
    listener: TcpListener,
    addr: SocketAddr,
    factory: String,
    tls_acceptor: Option<TlsAcceptor>,
    rx_listener: Receiver<Msg2Listener>,
    pub tx_listener: Sender<Msg2Listener>,
    tx_portal: Sender<Msg2Portal>
}

impl Listener {
    pub fn new(listener: TcpListener, addr: SocketAddr, tls_acceptor: Option<TlsAcceptor>, listen_id: String, tx_portal: Sender<Msg2Portal>, factory: &String) -> Self {
        let (tx_listener, rx_listener) = channel(50);
        Self {
            listen_id,
            tx_portal,
            factory: factory.clone(),
            tls_acceptor,
            listener,
            addr,
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

                                        self.tx_portal.send(Msg2Portal::AcceptTLS(tls_stream, addr, self.factory.clone())).await;
                                    } else {
                                        // Not sure what to do if TLS fails...
                                    }
                                },
                                Option::None => {
                                    // TLS is not engaged.
                                    self.tx_portal.send(Msg2Portal::AcceptTCP(tcp_stream, addr, self.factory.clone())).await;
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

pub enum Msg2Factory {
    AcceptTCP(String, TcpStream, SocketAddr),
    AcceptTLS(String, TlsStream<TcpStream>, SocketAddr),
    Kill
}

pub struct FactoryLink {
    pub factory_id: String,
    pub tx_factory: Sender<Msg2Factory>
}

pub struct ListenerLink {
    pub listen_id: String,
    pub factory: String,
    pub addr: SocketAddr,
    pub tls: bool,
    pub handle: JoinHandle<()>,
    pub tx_listener: Sender<Msg2Listener>,
}

pub struct ConnectionLink {
    pub addr: SocketAddr,
    pub conn_id: String,
    pub protocol: String,
    pub tx_protocol: Sender<Msg2Connection>,
}

#[derive(Clone)]
pub struct ClientInfo {
    pub conn_id: String,
    pub addr: SocketAddr,
    pub tls: bool,
    pub protocol: String,
    pub tx_protocol: Sender<Msg2Connection>
}

#[derive(Clone)]
pub struct ClientCapabilities {
    pub text: bool,
    pub utf8: bool,
    pub html: bool,
    pub mxp: bool,
    pub data: bool,
    pub ansi: bool,
    pub xterm256: bool,
    pub width: u16,
    pub height: u16,
    pub screen_reader: bool
}

pub enum Msg2Portal {
    Kill,
    DisconnectClient(String, Option<String>),
    AcceptTCP(TcpStream, SocketAddr, String),
    AcceptTLS(TlsStream<TcpStream>, SocketAddr, String),
    NewLink(ConnectionLink)
}


pub struct Portal {
    listeners: HashMap<String, ListenerLink>,
    connections: HashMap<String, ConnectionLink>,
    used_ids: HashSet<String>,
    factories: HashMap<String, FactoryLink>,
    pub tx_portal: Sender<Msg2Portal>,
    rx_portal: Receiver<Msg2Portal>,
    tx_sessmanager: Sender<Msg2SessionManager>
}

impl Portal {

    pub fn new(tx_portal: Sender<Msg2Portal>, rx_portal: Receiver<Msg2Portal>, tx_sessmanager: Sender<Msg2SessionManager>) -> Self {

        Self {
            listeners: Default::default(),
            connections: Default::default(),
            used_ids: Default::default(),
            factories: Default::default(),
            tx_portal,
            rx_portal,
            tx_sessmanager
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_portal.recv().await {
                match msg {
                    Msg2Portal::Kill => {
                        // This should full stop all listeners and clients and tasks then end this tasks.
                        for (_k, v) in self.listeners.iter_mut() {
                            v.tx_listener.send(Msg2Listener::Kill).await;
                        }
                        for (_k, v) in self.connections.iter_mut() {
                            v.tx_protocol.send(Msg2Connection::Kill).await;
                        }
                        break;
                    },
                    Msg2Portal::DisconnectClient(conn_id, reason) => {
                        // the Portal has been instructed to arbitrarily terminate one of its clients.
                    },
                    Msg2Portal::AcceptTCP(stream, addr, factory) => {
                        let new_id = self.generate_id();
                        if let Some(factory) = self.factories.get_mut(&factory) {
                            factory.tx_factory.send(Msg2Factory::AcceptTCP(new_id, stream, addr)).await;
                        }
                    },
                    Msg2Portal::AcceptTLS(stream, addr, factory) => {
                        let new_id = self.generate_id();
                        if let Some(factory) = self.factories.get_mut(&factory) {
                            factory.tx_factory.send(Msg2Factory::AcceptTLS(new_id, stream, addr)).await;
                        }
                    },
                    Msg2Portal::NewLink(link) => {
                        self.connections.insert(link.conn_id.clone(), link);
                    }
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
        let addr = listener.local_addr().unwrap();

        let tls_bool = tls.is_some();

        let mut listener = Listener::new(listener, addr.clone(), tls.clone(), listen_id.clone(), self.tx_portal.clone(), protocol);
        let tx_listener = listener.tx_listener.clone();

        let handle = tokio::spawn(async move {listener.run().await});

        let mut listen_link = ListenerLink {
            addr,
            listen_id: listen_id.clone(),
            factory: protocol.clone(),
            handle,
            tx_listener,
            tls: tls_bool
        };
        self.listeners.insert(listen_id, listen_link);
    }

    fn generate_id(&mut self) -> String {
        loop {
            let new_id: String = random_alphanum(16);
            if !self.used_ids.contains(&new_id) {
                self.used_ids.insert(new_id.clone());
                return new_id;
            }
        }
    }
}

// Thermite lib does not actually implement a Session Manager. Just the means by which protocols can
// communicate with a SessionManager and Sessions.
pub enum Msg2SessionManager {
    Kill,
    ClientReady(String, ClientInfo, ClientCapabilities),
    ClientDisconnected(Option<String>),
}

pub enum Msg2Session {
    Kill,
    ClientCommand(String),
    ClientDisconnected(Option<String>),
    ClientCapabilities(ClientCapabilities)
}