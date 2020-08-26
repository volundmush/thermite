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

use thermite_lib::{
    telnet::TelnetProtocol,
    websocket::WebSocketProtocol
};


pub enum ProtocolType {
    Telnet,
    WebSocket
}

pub enum Msg2Listener {
    Kill
}

pub struct Listener {
    pub listen_id: String,
    pub listen_type: ProtocolType,
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
            listen_type: protocol,
            tx_portal,
            tls_acceptor,
            listener,
            addr,
            protocol,
            tx_listener,
            rx_listener
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
    pub fn new(conn_id: String, conn: impl AsyncRead + AsyncWrite + Send + 'static, addr: SocketAddr, tls: bool, tx_portal: Sender<Ms2Portal>, protocol: ProtocolType) -> Self {
        let (tx_protocol, rx_protocol) = channel(50);

        match protocol {
            ProtocolType::Telnet => {
                let mut tel_prot = TelnetProtocol::new(conn_id.clone(), conn, addr.clone(), tls, rx_protocol, tx_portal.clone());
                let handle = tokio::spawn(async move {protocol.run().await;});
            },
            ProtocolType::WebSocket => {
                // Will fix this up later...
            }
        }
        Self {
            addr,
            conn_id,
            protocol,
            handle,
            tx_protocol
        }        
    }
}



pub enum Msg2Portal {
    Kill,
    DisconnectClient(String, Option<String>),
    NewTcpConnection(TcpStream, SocketAddr, ProtocolType),
    NewTlsConnection(TlsStream<TcpStream>, SocketAddr, ProtocolType)
}

pub struct ClientPortal {
    pub listeners: HashMap<String, ListenerLink>,
    pub connections: HashMap<String, ConnectionLink>,
    pub tx_portal: Sender<Msg2Portal>,
    pub rx_portal: Receiver<Msg2Portal>,
}

impl ClientPortal {
    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_portal.recv().await {
                match msg {
                    Msg2Portal::Kill => {
                        // This should full stop all listeners and clients and tasks then end this tasks.
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
                    }
                }
            }
        }
    }

    pub fn listen(&mut self, listen_id: String, listener: TcpListener, tls: Option<TlsAcceptor>, protocol: ProtocolType) {
        if self.listeners.contains_key(&listen_id) {
            return;
        }
        let addr = listener.local_addr().unwrap();

        let mut listener = Listener::new(listener, addr.clone(), tls, listen_id.clone(), self.tx_portal.clone(), protocol.clone());
        let tx_listener = listener.tx_listener.clone();

        let handle = tokio::spawn(async move {listener.run().await});

        let mut listen_stub = ListenerLink {
            addr,
            listen_id: listen_id.clone(),
            listen_type: protocol,
            handle,
            tx_listener,
            tls
        };
        self.listeners.insert(listen_id, listen_stub);
    }

    fn accept(&mut self, conn: impl AsyncRead + AsyncWrite + Send + 'static, addr: SocketAddr, tls: bool, protocol: ProtocolType) {
        let new_id = self.generate_id();
        let conn_data = ConnectionLink::new(new_id.clone(), conn, addr, tls, self.tx_portal.clone(), protocol);
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