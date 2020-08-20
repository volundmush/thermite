use tokio::prelude::*;
use bytes::Bytes;
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use tokio::sync::mpsc;
use std::net::{SocketAddr};



pub struct TelnetServer {
    connections:
}


pub enum MsgManagerToConnection {
    Data(Bytes),
    Registered,
}

impl Message for MsgManagerToConnection {
    type Result = ();
}

pub enum MsgConnectionToManager {
    Register(Connection),
    ConnectionLost(Uuid, String),
    UserCommand(Uuid, String)
}

impl Message for MsgConnectionToManager {
    type Result = ();
}

pub enum Protocol {
    Telnet,
}

pub struct Connection {
    pub uuid: Uuid,
    pub addr: Recipient<MsgManagerToConnection>,
    pub protocol: Protocol,
}


#[derive(Default)]
pub struct NetworkManager {
    connections: HashMap<Uuid, Connection>,
}

impl Actor for NetworkManager {
    type Context = Context<Self>;
}

impl Handler<MsgConnectionToManager> for NetworkManager {
    type Result = ();

    fn handle(&mut self, msg: MsgConnectionToManager, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            MsgConnectionToManager::Register(conn) => {
                println!("REGISTER CONNECTION: {}", conn.uuid);
                self.register_connection(conn, ctx);
            }
            MsgConnectionToManager::UserCommand(uuid, comm) => {
                self.connections.get_mut(&uuid).unwrap().addr.do_send(MsgManagerToConnection::Data(Bytes::from(format!("APP ECHO: {}", comm))));
            }
            MsgConnectionToManager::ConnectionLost(uuid, reason) => {
                self.connections.remove(&uuid);
                println!("LOST CONNECTION: {}", uuid);
            }
        }
    }
}

impl NetworkManager {
    fn register_connection(&mut self, conn: Connection, ctx: &mut Context<Self>) {
        let uuid = conn.uuid.clone();
        self.connections.insert(uuid.clone(), conn);
        self.connections.get_mut(&uuid).unwrap().addr.do_send(MsgManagerToConnection::Registered);
    }
}