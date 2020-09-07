use std::{
    collections::{HashMap, HashSet},
    io,
    vec::Vec,
    net::SocketAddr,
    time::Duration,
    sync::Arc
};

use tokio::{
    prelude::*,
    sync::mpsc::{Receiver, Sender, channel},
    time
};

use tokio_util::codec::{Framed};

use bytes::{BytesMut, Bytes, BufMut, Buf};

use futures::{
    sink::{SinkExt},
    stream::{StreamExt}
};

use serde_json::Value as JsonValue;

use thermite_net::{Msg2Factory, FactoryLink};

use thermite_telnet::{
    codes as tc,
    codec::TelnetCodec,
    protocol::{
        Msg2MudTelnetProtocol, Msg2TelnetProtocol, TelnetConfig, TelnetProtocol
    }
};
use thermite_util::text::generate_id;

use crate::{Msg2ProtocolManager, Msg2MudProtocol, ProtocolLink,
            ProtocolCapabilities};

impl From<TelnetConfig> for ProtocolCapabilities {
    fn from(src: ProtocolCapabilities) -> Self {
        ProtocolCapabilities {
            client_name: src.client_name,
            client_version: src.client_version,
            utf8: src.utf8,
            html: false,
            mxp: src.mxp,
            msdp: src.msdp,
            gmcp: src.gmcp,
            ansi: src.ansi,
            xterm256: src.xterm256,
            width: src.width,
            height: src.height,
            screen_reader: src.screen_reader
    }
}


pub struct MudTelnetProtocol {
    // This serves as a higher-level actor that abstracts a bunch of the lower-level
    // nitty-gritty so the Session doesn't need to deal with it.
    conn_id: String,
    addr: SocketAddr,
    tls: bool,
    config: TelnetConfig,
    active: bool,
    sent_link: bool,
    pub tx_protocol: Sender<Msg2MudProtocol>,
    rx_protocol: Receiver<Msg2MudProtocol>,
    pub tx_mud: Sender<Msg2MudTelnetProtocol>,
    rx_mud: Receiver<Msg2MudTelnetProtocol>,
    tx_telnet: Sender<Msg2TelnetProtocol>,
    tx_manager: Sender<Msg2ProtocolManager>,
    running: bool
}


impl MudTelnetProtocol {
    pub fn new(conn_id: String, addr: SocketAddr, tls: bool, tx_manager: Sender<Msg2ProtocolManager>, 
        tx_telnet: Sender<Msg2TelnetProtocol>, tx_mud: Sender<Msg2MudTelnetProtocol>, rx_mud: Receiver<Msg2MudTelnetProtocol>) -> Self {

        let (tx_protocol, rx_protocol) = channel(50);

        Self {
            conn_id,
            addr,
            tls,
            config: TelnetConfig::default(),
            handshakes_left,
            tx_protocol,
            rx_protocol,
            tx_manager,
            active: false,
            sent_link: false,
            tx_telnet,
            tx_mud,
            rx_mud,
            running: true
        }
    }

    fn link(&self) -> ProtocolLink {
        ProtocolLink {
            conn_id: self.conn_id.clone(),
            addr: self.addr.clone(),
            tls: self.tls.clone(),
            capabilities: ProtocolCapabilities::from(self.config.clone()),
            tx_protocol: self.tx_protocol.clone()
        }
    }

    async fn check_ready(&mut self) {
        if self.active || self.sent_link {
            return;
        }
        if self.handshakes_left.len() == 0 && !self.ttype_pending {
            let _ = self.tx_manager.send(Msg2ProtocolManager::NewProtocol(self.link())).await;
            self.sent_link = true;
        }
    }

    async fn get_ready(&mut self) {
        if self.active || self.sent_link {
            return;
        }
        let _ = self.tx_manager.send(Msg2ProtocolManager::NewProtocol(self.link())).await;
        self.sent_link = true;
    }

    pub async fn run(&mut self) {

        while self.running {
            tokio::select! {
                t_msg = self.rx_mud.recv() => {
                    if let Some(msg) = t_msg {
                        let _ = self.process_mudtelnet_message(msg).await;
                        
                    } else {
                        if self.sent_link {
                                    let _ = self.tx_manager.send(Msg2ProtocolManager::ProtocolDisconnected(self.conn_id.clone())).await;
                                }
                        break;
                    }
                },
                p_msg = self.rx_protocol.recv() => {
                    if let Some(msg) = p_msg {
                        println!("Got Protocol Message: {:?}", msg);
                        match msg {
                            Msg2MudProtocol::Disconnect => {
                                break;
                            },
                            Msg2MudProtocol::Line(text) => {
                                let _ = self.conn.send(TelnetEvent::Line(text)).await;
                            },
                            Msg2MudProtocol::Prompt(text) => {

                            },
                            Msg2MudProtocol::OOB(String, JsonValue) => {

                            },
                            Msg2MudProtocol::MSSP => {

                            },
                            Msg2MudProtocol::Ready => {
                                self.active = true;
                            }
                            Msg2MudProtocol::GetReady => {
                                let _ = self.get_ready().await;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn process_mudtelnet_message(&mut self, msg: Msg2MudTelnetProtocol) {
        match msg {
            Msg2MudTelnetProtocol::Line(text) => self.receive_line(text).await,
            Msg2MudTelnetProtocol::Config(config) => self.receive_config(config).await,
            Msg2MudTelnetProtocol::Data(command, json) => self.receive_data(command, data).await,
            Msg2MudTelnetProtocol::Ready(config) => {},
            Msg2MudTelnetProtocol::ClientDisconnected => {}            
        }
    }

    async fn receive_line(&mut self, text: String) {
        // No commands will be sent until the Protocol has been recognized by the ProtocolManager.
        println!("RECEIVED LINE: {}", text);
        if self.active {
            let _ = self.tx_manager.send(Msg2ProtocolManager::ProtocolCommand(self.conn_id.clone(), text)).await;
        } else {
            println!("But we're not active.");
        }
    }
}

pub struct TelnetProtocolFactory {
    factory_id: String,
    pub tx_factory: Sender<Msg2Factory>,
    rx_factory: Receiver<Msg2Factory>,
    telnet_options: Arc<HashMap<u8, TelnetOption>>,
    telnet_statedef: HashMap<u8, TelnetOptionState>,
    opening_bytes: Bytes,
    tx_manager: Sender<Msg2ProtocolManager>,
    ids: HashSet<String>,
}

impl TelnetProtocolFactory {
    pub fn new(factory_id: String, options: HashMap<u8, TelnetOption>, tx_manager: Sender<Msg2ProtocolManager>) -> Self {
        let (tx_factory, rx_factory) = channel(50);

        // Since this only needs to be done once... we'll clone it from here.
        let mut opstates: HashMap<u8, TelnetOptionState> = Default::default();

        let mut raw_bytes = BytesMut::with_capacity(options.len() * 3);

        for (b, handler) in options.iter() {
            let mut state = TelnetOptionState::default();

            if handler.start_local {
                state.local.negotiating = true;
                raw_bytes.put_u8(tc::IAC);
                raw_bytes.put_u8(tc::WILL);
                raw_bytes.put_u8(b.clone());
            }
            if handler.start_remote {
                state.remote.negotiating = true;
                raw_bytes.put_u8(tc::IAC);
                raw_bytes.put_u8(tc::DO);
                raw_bytes.put_u8(b.clone());
            }
            opstates.insert(b.clone(), state);
        }

        Self {
            factory_id,
            tx_factory,
            rx_factory,
            telnet_options: Arc::new(options),
            telnet_statedef: opstates,
            opening_bytes: raw_bytes.freeze(),
            tx_manager,
            ids: HashSet::default()
        }
    }

    pub fn link(&self) -> FactoryLink {
        FactoryLink {
            factory_id: self.factory_id.clone(),
            tx_factory: self.tx_factory.clone()
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(f_msg) = self.rx_factory.recv().await {
                match f_msg {
                    Msg2Factory::AcceptTLS(stream, addr) => {
                        self.accept(stream, addr, true);
                    },
                    Msg2Factory::AcceptTCP(stream, addr) => {
                        self.accept(stream, addr, false);
                    },
                    Msg2Factory::Kill => {
                        break;
                    }
                }
            }
        }
    }

    fn accept<C>(&mut self, conn: C, addr: SocketAddr, tls: bool)
        where C: AsyncRead + AsyncWrite + Send + 'static + Unpin + std::marker::Sync
    {
        let telnet_codec = Framed::new(conn, TelnetCodec::new(TelnetConnectionType::Server, TelnetConnectionMode::Mud, 8192));
        let gen_id = generate_id(12, &self.ids);
        let conn_id = format!("{}_{}", self.factory_id, gen_id);
        self.ids.insert(gen_id);

        let (tx_mud, rx_mud) = channel(50);
        let (tx_telnet, rx_telnet) = channel(50);

        let mut tel_prot = TelnetProtocol::new(telnet_codec, addr.clone(), tls.clone(), tx_mud.clone(), tx_telnet.clone(), rx_telnet, self.telnet_options.clone(), self.telnet_statedef.clone());
        
        let mut mud_tel = MudTelnetProtocol::new(conn_id, addr, tls, self.tx_manager.clone(), tx_telnet, tx_mud, rx_mud);
        
        let opening = self.opening_bytes.clone();
        let _ = tokio::spawn(async move {tel_prot.run(opening).await;});
        let _ = tokio::spawn(async move {mud_tel.run().await;});
    }
}