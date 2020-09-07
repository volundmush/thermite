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

// Feed one of these to the Portal to implement a connection filter.
#[async_trait::async_trait]
pub trait PortalFilter {
    async fn check(&mut self, addr: &SocketAddr) -> bool;
}

pub enum Msg2Listener {
    Kill
}


pub struct Listener {
    listen_id: String,
    listener: TcpListener,
    factory: String,
    tls_acceptor: Option<TlsAcceptor>,
    rx_listener: Receiver<Msg2Listener>,
    pub tx_listener: Sender<Msg2Listener>,
    tx_portal: Sender<Msg2Portal>
}

impl Listener {
    pub fn new(listener: TcpListener, tls_acceptor: Option<TlsAcceptor>, listen_id: String, tx_portal: Sender<Msg2Portal>, factory: &String) -> Self {
        let (tx_listener, rx_listener) = channel(50);
        Self {
            listen_id,
            tx_portal,
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

                                        let _ = self.tx_portal.send(Msg2Portal::AcceptTLS(tls_stream, addr, self.factory.clone())).await;
                                    } else {
                                        // Not sure what to do if TLS fails...
                                    }
                                },
                                Option::None => {
                                    // TLS is not engaged.
                                    let _ = self.tx_portal.send(Msg2Portal::AcceptTCP(tcp_stream, addr, self.factory.clone())).await;
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
    AcceptTCP(TcpStream, SocketAddr),
    AcceptTLS(TlsStream<TcpStream>, SocketAddr),
    Kill
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
    AcceptTCP(TcpStream, SocketAddr, String),
    AcceptTLS(TlsStream<TcpStream>, SocketAddr, String)
}


pub struct Portal {
    listeners: HashMap<String, ListenerLink>,
    factories: HashMap<String, FactoryLink>,
    pub tx_portal: Sender<Msg2Portal>,
    rx_portal: Receiver<Msg2Portal>
}

impl Portal {

    pub fn new(filter: Option<Box<dyn PortalFilter>>) -> Self {

        let (tx_portal, rx_portal) = channel(50);

        Self {
            listeners: Default::default(),
            factories: Default::default(),
            filter,
            tx_portal,
            rx_portal
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_portal.recv().await {
                match msg {
                    Msg2Portal::Kill => {
                        // This should full stop all listeners and clients and tasks then end this tasks.
                        for (_k, v) in self.listeners.iter_mut() {
                            let _ = v.tx_listener.send(Msg2Listener::Kill).await;
                        }
                        for (_k, v) in self.factories.iter_mut() {
                            let _ = v.tx_factory.send(Msg2Factory::Kill).await;
                        }
                        break;
                    },
                    Msg2Portal::AcceptTCP(stream, addr, factory) => {
                        if let Some(factory) = self.factories.get_mut(&factory) {
                            let _ = factory.tx_factory.send(Msg2Factory::AcceptTCP(stream, addr)).await;
                        }
                    },
                    Msg2Portal::AcceptTLS(stream, addr, factory) => {
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

        let mut listener = Listener::new(listener, tls.clone(), listen_id.clone(), self.tx_portal.clone(), protocol);
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