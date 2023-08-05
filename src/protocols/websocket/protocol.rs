use std::{
    collections::{HashMap, HashSet},
    vec::Vec,
    net::SocketAddr,
    time::{Duration, Instant},
};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc::{Receiver, Sender, channel},
    time
};

use tokio_util::codec::{Framed};

use tokio_stream::wrappers::IntervalStream;

use bytes::{BytesMut, Bytes, BufMut, Buf};

use futures::{
    sink::{SinkExt},
    stream::{StreamExt}
};

use serde_json::{json, Value as JsonValue};

use once_cell::sync::Lazy;

use crate::{
    protocols::{
        {ProtocolCapabilities, Color, ProtocolLink, MudData}
    },
    msg::{Msg2MudProtocol, Msg2Portal, Msg2PortalFromClient},
    util::ensure_crlf,
    IS_TLS_ENABLED,
    TX_PORTAL
};

use warp::ws::{WebSocket, Message};
use crate::protocols::Protocol;

pub struct WebsocketProtocol {
    conn_id: usize,
    config: ProtocolCapabilities,
    tx_portal: Sender<Msg2Portal>,
    tx_protocol: Sender<Msg2MudProtocol>,
    rx_protocol: Receiver<Msg2MudProtocol>,
    running: bool,
    time_created: Instant,
    time_activity: Instant,
    conn: WebSocket
}

impl WebsocketProtocol {
    pub fn new(conn_id: usize, conn: WebSocket, addr: String, port: u16, hostnames: Vec<String>) -> Self {
        let (tx_protocol, rx_protocol) = channel(10);

        let tx_portal = TX_PORTAL.lock().unwrap().clone().unwrap();

        let mut out = Self {
            conn_id,
            config: Default::default(),
            tx_portal,
            tx_protocol,
            rx_protocol,
            running: false,
            time_created: Instant::now(),
            time_activity: Instant::now(),
            conn
        };

        out.config.protocol = Protocol::WebSocket;
        out.config.tls = *IS_TLS_ENABLED.lock().unwrap();
        out.config.host_address = addr;
        out.config.host_port = port;
        out.config.color = Color::TrueColor;
        out.config.client_name = "Thermite Webclient".to_string();
        out.config.client_version = "0.1".to_string();
        out.config.utf8 = true;

        out

    }

    fn make_link(&self) -> ProtocolLink {
        ProtocolLink {
            conn_id: self.conn_id,
            capabilities: self.config.clone(),
            tx_protocol: self.tx_protocol.clone()
        }
    }

    pub async fn run(&mut self) {
        // This'll be used for sending pings.
        let mut interval_timer = IntervalStream::new(time::interval(Duration::from_millis(100)));

        self.running = true;

        let link = self.make_link();

        // Unlike telnet, we go live immediately with websockets. YAHOO.
        let _ = self.tx_portal.send(Msg2Portal::ClientConnected(link)).await;

        // The main loop which operates the protocol during and after negotiation.
        while self.running {
            tokio::select! {
            t_msg = self.conn.next() => self.handle_conn(t_msg).await,
            p_msg = self.rx_protocol.recv() => {
                if let Some(msg) = p_msg {
                    let _ = self.process_protocol_message(msg).await;
                }
            },
            i_msg = interval_timer.next() => {
                if let Some(ins) = i_msg {
                    let _ = self.handle_interval_timer(ins.into_std()).await;
                    }
                }
            }
        }
    }

    async fn handle_interval_timer(&mut self, ins: Instant) {
        // send a ping over self.conn.
        let _ = self.conn.send(Message::ping(vec![])).await;
    }

    async fn process_protocol_message(&mut self, msg: Msg2MudProtocol) {
        match msg {
            Msg2MudProtocol::Disconnect => {
                self.running = false;
            },
            Msg2MudProtocol::Data(v) => {
                for d in v {
                    let _ = self.process_protocol_message_data(d).await;
                }
            }
        }
    }

    async fn handle_conn(&mut self, t_msg: Option<Result<Message, warp::Error>>) {
        if let Some(msg) = t_msg {
            match msg {
                Ok(msg) => {
                    if msg.is_text() {
                        let _ = self.handle_text_message(msg.to_str().unwrap()).await;
                    } else if msg.is_binary() {
                        let _ = self.handle_binary_message(msg.into_bytes());
                    } else if msg.is_ping() {
                        let _ = self.conn.send(Message::pong(msg.into_bytes())).await;
                    } else if msg.is_close() {
                        let _ = self.tx_portal.send(Msg2Portal::ClientDisconnected(self.conn_id, String::from("dunno yet"))).await;
                        self.running = false;
                    }
                },
                Err(e) => {
                    println!("Error reading from websocket: {}", e);
                    let _ = self.tx_portal.send(Msg2Portal::ClientDisconnected(self.conn_id, String::from("dunno yet"))).await;
                    self.running = false;
                }
            }
        } else {
            // end of stream, it closed unexpectedly.
            let _ = self.tx_portal.send(Msg2Portal::ClientDisconnected(self.conn_id, String::from("dunno yet"))).await;
            self.running = false;
        }
    }

    async fn process_protocol_message_data(&mut self, d: MudData) {
        // One of the great things about the webclient is...
        // IT will handle this. We don't need to do anything. Much.

        // Serialize d using serde into a json object and send it out as text.
        // we must create a new JSON Array and send the data like this:
        // [d.cmd, d.args, d.kwargs]
        // Example: ["text", ["Hello world!"], {}]
        // This will allow us to match the Evennia webclient format.

        // REAL CODE GOES HERE...
        let data = json!([d.cmd, d.args, d.kwargs]);

        // Serialize data into a JSON string
        let data_string = match serde_json::to_string(&data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialize data into JSON: {}", e);
                return;
            }
        };

        let _ = self.conn.send(Message::text(data_string)).await;

    }

    async fn handle_text_message(&mut self, s: &str) {
        // Any text message we receive should be a json object that can become a MudData.
        // Deserialize it and send it to the portal.
        if let Ok(d) = serde_json::from_str::<MudData>(s) {
            let _ = self.tx_portal.send(Msg2Portal::FromClient(self.conn_id, Msg2PortalFromClient::Data(vec![d]))).await;
        }

    }

    async fn handle_binary_message(&mut self, v: Vec<u8>) {

    }

}