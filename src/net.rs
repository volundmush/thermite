use actix::prelude::*;
use bytes::Bytes;
use std::collections::HashMap;
use uuid::Uuid;

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
    UserCommand(String)
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

    }
}

impl NetworkManager {
    fn register_connection(&mut self, conn: Connection, ctx: &mut Context<Self>) {
        let uuid = conn.uuid.clone();
        self.connections.insert(uuid.clone(), conn);
        self.connections.get_mut(&uuid).unwrap().addr.do_send(MsgManagerToConnection::Registered);
    }
}