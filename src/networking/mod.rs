use std::collections::HashMap;
use actix::prelude::*;

pub mod telnet;


pub trait GameProtocol {
    async fn send_bytes(&mut self, data: &[u8], size: usize);
    async fn receive_bytes(&mut self, data: &[u8], size: usize);

    async fn start(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }
}

pub struct HostInfo {
    pub address: String,
    // probably more stuff...
}


pub struct GameConnection {
    pub uuid: uuid::Uuid,
    pub host: HostInfo,
}



pub struct GameServer {
    pub name: String,
    pub address: String,
    pub port: u32,
}


pub trait ConnectionHandler {

    async fn start(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), std::io::Error> {

        Ok(())
    }

    async fn connection_loop() {
        let mut incoming = listener.incoming();

        let mut running: bool = true;

        let mut accept: bool = true;

        while running {
            tokio::select! {
                Some(socket_res) = incoming.next(), if accept => {
                        // If there's a new connection AND we are being allowed to accept them right now...
                        match socket_res {
                            Ok(socket) => {
                                println!("Accepted connection from {:?}", socket.peer_addr());

                                let mut connection = handler.accept_connection(socket);
                                // Something about sending an alert of the
                            }
                            Err(err) => {
                                println!("accept error = {:?}", err);
                            }
                        }
                    }
                Some(msg) = chan_from_manager.recv() => {
                        // actually have no idea how this works... checking...
                }
            }
        }
    }

    // Not sure of a better way to get a 'could be anything darnit' codec implementation working yet.
    fn new_protocol() -> Box<dyn GameProtocol>;

    async fn accept_connection(&mut self, mut socket: net::TcpStream, name: String) -> GameConnection {
        let mut info = HostInfo {
            address: socket.peer_addr()
        };

        let (from_tx, from_rx) = channel<FromConnectionToManager>(100);
        let (to_tx, to_rx) = channel<FromManagerToConnection>(100);

        // yeah I need some more async setup wizardry here...
        let mut protocol = self.new_protocol();

        let task = tokio::spawn(async move {
            ConnectionManager.connection_loop().await;
        });

        // Then I create the struct and return it?
        GameConnection {
            uuid: uuid::Uuid::new_v4(),
            host: info,
        }

    }
}

pub enum FromConnectionToManager {
    Connected(GameConnection),
    Disconnected(uuid::Uuid)
}

pub enum FromManagerToConnection {

}

pub enum FromEngineToManager {
    Start,
    Stop,
    Shutdown,
    Kick((uuid::Uuid, String))
}

pub enum FromManagerToEngine {
    Disconnected(uuid::Uuid),
    Connected(uuid::Uuid)
}

pub struct ConnectionManager {
    pub servers: HashMap<String, GameServer>,
    pub connections: HashMap<uuid::Uuid, GameConnection>,
    pub enabled: bool,
    pub channel_from_engine: Receiver<FromEngineToManager>,
    pub channel_to_engine: Sender<FromManagerToEngine>
}

impl Actor for ConnectionManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {

    }
}

impl ConnectionManager {
    async fn start(&mut self) -> Result<(), std::io::Error> {
        if self.enabled {
            Ok(())
        }
        let address: String = String::from("10.0.0.226");
        let port: u32 = 4200;
        let mut telnet_server = self.start_server(address, port, String::from("telnet"), TelnetConnectionHandler {}).await?;
        self.servers.insert(String::from("telnet"), telnet_server);
        self.enabled = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), std::io::Error> {
        if !self.enabled {
            Ok(())
        }
        Ok(())
    }

    async fn server_loop(listener: net::TcpListener, chan_to_manager: Sender<FromServerToManager>, chan_from_manager: Receiver<FromManagerToServer>,
                         name: String, mut handler: impl ConnectionHandler) {
        let mut incoming = listener.incoming();

        let mut running: bool = true;

        let mut accept: bool = true;

        while running {
            tokio::select! {
                _ = incoming.next(), if accept => {
                        // If there's a new connection AND we are being allowed to accept them right now...
                        match _ {
                            Ok(socket) => {
                                println!("Accepted connection from {:?}", socket.peer_addr());

                                let mut connection = handler.accept_connection(socket, name.clone());
                                // Something about sending an alert of the
                            }
                            Err(err) => {
                                println!("accept error = {:?}", err);
                            }
                        }
                    }
                _ = chan_from_manager.recv() => {
                        // actually have no idea how this works... checking...
                }
            }
        }

        while let Some(socket_res) = incoming.next().await {

        }
    }

    async fn connection_loop() {

    }

    async fn start_server(&mut self, address: String, port: u32, name: String, mut handler: impl ConnectionHandler) -> Result<ServerHolder, std::io::Error> {
        let addr: String = format!("{}:{}", address, port);

        // If not for this, I wouldn't need a Result. but binding can technically fail...
        let mut listener = net::TcpListener::bind(addr).await?;

        let (from_tx, from_rx) = tokio::sync::mspc::channel<FromServerToManager>(100);
        let (to_tx, to_rx) = tokio::sync::mspc::channel<FromManagerToServer>(100);

        // Everything between here and line 130? I got no idea.
        let mut task = tokio::spawn(async move {
            // not sure if I actually NEED to await here...
            ConnectionManager.server_loop(listener, to_rx, from_tx, name.clone(), handler).await;
        });

        let mut server = GameServer {
            name,
            address,
            port,
        };
        Ok(server)
    }
}
