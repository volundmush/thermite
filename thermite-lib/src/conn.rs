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

use crate::{
    telnet::TelnetProtocol,
    websocket::WebSocketProtocol,
    random_alphanum
};


#[derive(Clone)]
pub enum ProtocolType {
    Telnet,
    WebSocket
}


pub enum Msg2Protocol {
    Kill,
    Disconnect(Option<String>),
    Ready(Sender<Msg2Session>)
}


pub enum Msg2Listener {
    Kill
}


pub struct Listener {
    pub listen_id: String,
    pub listener: TcpListener,
    pub addr: SocketAddr,
    pub protocol: ProtocolType,
    pub tls_acceptor: Option<TlsAcceptor>,
    pub rx_listener: Receiver<Msg2Listener>,
    pub tx_listener: Sender<Msg2Listener>,
    pub tx_portal: Sender<Msg2Portal>
}

impl Listener {
    pub fn new(listener: TcpListener, addr: SocketAddr, tls_acceptor: Option<TlsAcceptor>, listen_id: String, tx_portal: Sender<Msg2Portal>, protocol: ProtocolType) -> Self {
        let (tx_listener, rx_listener) = channel(50);
        Self {
            listen_id,
            tx_portal,
            protocol,
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

                                        self.tx_portal.send(Msg2Portal::NewTlsConnection(tls_stream, addr, self.protocol.clone())).await;
                                    } else {
                                        // Not sure what to do if TLS fails...
                                    }
                                },
                                Option::None => {
                                    // TLS is not engaged.
                                    self.tx_portal.send(Msg2Portal::NewTcpConnection(tcp_stream, addr, self.protocol.clone())).await;
                                }
                            }
                        },
                        Err(e) => {

                        }
                    }
                },
            }
        }
    }

}

pub struct ListenerLink {
    pub listen_id: String,
    pub listen_type: ProtocolType,
    pub addr: SocketAddr,
    pub tls: bool,
    pub handle: JoinHandle<()>,
    pub tx_listener: Sender<Msg2Listener>,
}

pub struct ConnectionLink {
    pub addr: SocketAddr,
    pub conn_id: String,
    pub protocol: ProtocolType,
    pub handle: JoinHandle<()>,
    pub tx_protocol: Sender<Msg2Protocol>,
}

impl ConnectionLink {
    pub fn new(conn_id: String, conn: impl AsyncRead + AsyncWrite + Send + 'static + Unpin,
               addr: SocketAddr, tls: bool, tx_portal: Sender<Msg2Portal>, protocol: ProtocolType,
               tx_sessmanager: Sender<Msg2SessionManager>) -> Self {
        let (tx_protocol, rx_protocol) = channel(50);

        match protocol {
            ProtocolType::Telnet => {
                let mut tel_prot = TelnetProtocol::new(conn_id.clone(),
                                                       conn, addr.clone(), tls, tx_protocol.clone(), rx_protocol,
                                                       tx_portal.clone(), tx_sessmanager);
                let handle = tokio::spawn(async move {tel_prot.run().await;});

                Self {
                    addr,
                    conn_id,
                    protocol,
                    handle,
                    tx_protocol
                }

            },
            ProtocolType::WebSocket => {
                let mut web_prot = WebSocketProtocol {};
                let handle = tokio::spawn(async move {web_prot.run().await});

                Self {
                    addr,
                    conn_id,
                    protocol,
                    handle,
                    tx_protocol
                }

            }
        }

    }
}

#[derive(Clone)]
pub struct ClientInfo {
    pub conn_id: String,
    pub addr: SocketAddr,
    pub tls: bool,
    pub protocol: ProtocolType,
    pub tx_protocol: Sender<Msg2Protocol>
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
    NewTcpConnection(TcpStream, SocketAddr, ProtocolType),
    NewTlsConnection(TlsStream<TcpStream>, SocketAddr, ProtocolType)
}

pub struct Portal {
    listeners: HashMap<String, ListenerLink>,
    connections: HashMap<String, ConnectionLink>,
    pub tx_portal: Sender<Msg2Portal>,
    rx_portal: Receiver<Msg2Portal>,
    tx_sessmanager: Sender<Msg2SessionManager>
}

impl Portal {

    pub fn new(tx_sessmanager: Sender<Msg2SessionManager>) -> Self {
        let (tx_portal, rx_portal) = channel(50);
        Self {
            listeners: Default::default(),
            connections: Default::default(),
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
                        for (k, v) in self.listeners.iter_mut() {
                            v.tx_listener.send(Msg2Listener::Kill).await;
                        }
                        for (k, v) in self.connections.iter_mut() {
                            v.tx_protocol.send(Msg2Protocol::Kill).await;
                        }
                        break;
                    },
                    Msg2Portal::DisconnectClient(conn_id, reason) => {
                        // the Portal has been instructed to arbitrarily terminate one of its clients.
                    },
                    Msg2Portal::NewTcpConnection(stream, addr, protocol) => {
                        self.accept(stream, addr, false, protocol);
                    },
                    Msg2Portal::NewTlsConnection(stream, addr, protocol) => {
                        self.accept(stream, addr, true, protocol);
                    },
                }
            }
        }
    }

    pub fn listen(&mut self, listen_id: String, listener: TcpListener, tls: Option<TlsAcceptor>, protocol: ProtocolType) {
        if self.listeners.contains_key(&listen_id) {
            return;
        }
        let addr = listener.local_addr().unwrap();

        let mut tls_bool = false;
        if let Some(tls_check) = &tls {
            tls_bool = true;
        }

        let mut listener = Listener::new(listener, addr.clone(), tls, listen_id.clone(), self.tx_portal.clone(), protocol.clone());
        let tx_listener = listener.tx_listener.clone();

        let handle = tokio::spawn(async move {listener.run().await});


        let mut listen_stub = ListenerLink {
            addr,
            listen_id: listen_id.clone(),
            listen_type: protocol,
            handle,
            tx_listener,
            tls: tls_bool
        };
        self.listeners.insert(listen_id, listen_stub);
    }

    fn accept(&mut self, conn: impl AsyncRead + AsyncWrite + Send + 'static + Unpin, addr: SocketAddr, tls: bool, protocol: ProtocolType) {
        let new_id = self.generate_id();
        let conn_data = ConnectionLink::new(new_id.clone(), conn, addr, tls, self.tx_portal.clone(), protocol, self.tx_sessmanager.clone());
        self.connections.insert(new_id, conn_data);
    }

    fn generate_id(&self) -> String {
        loop {
            let new_id: String = random_alphanum(16);
            if !self.connections.contains_key(&new_id) {
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