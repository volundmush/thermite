use tokio::{
    prelude::*,
    sync::mpsc::{Receiver, Sender, channel},
    sync::oneshot,
};

use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    error::Error
};

use thermite_protocol::{
    Msg2ProtocolManager,
    ProtocolCapabilities,
    ProtocolLink,
    Msg2MudProtocol,
    ConnectResponse
};

use regex::{Regex, Captures};

use crate::{
    db::Msg2DbManager,
    models::{User, Game, Board, Post, PostRead, Channel, ChannelSub, GameMember, MemberStorage},
    commands::{Command, CommandAction}
};

use chrono::NaiveDateTime;
use crate::protocol::Msg2ConnManager;
use tokio::macros::support::Future;


#[derive(Clone, Debug)]
pub struct ProtocolWrapper {
    pub link: ProtocolLink,
    pub account_id: Option<isize>,
    pub member: Option<isize>
}

impl From<ProtocolLink> for ProtocolWrapper {
    fn from(src: ProtocolLink) -> Self {
        Self {
            link: src,
            account_id: None,
            member: None
        }
    }
}

#[derive(Clone, Debug)]
pub struct UserWrapper {
    pub user: User,
    pub protocols: HashSet<String>,
    pub login_date: NaiveDateTime,
    pub last_check: Option<NaiveDateTime>,
}

impl From<User> for UserWrapper {
    fn from(user: User) -> Self {
        Self {
            user,
            protocols: Default::default(),
            login_date: Default::default(),
            last_check: None
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameWrapper {
    pub game: Game,
    pub protocols: HashSet<String>,
}

impl From<Game> for GameWrapper {
    fn from(game: Game) -> Self {
        Self {
            game,
            protocols: Default::default()
        }
    }
}

#[derive(Debug)]
pub struct ProgramState {
    protocols: HashMap<String, ProtocolWrapper>,
    users_online: HashMap<isize, UserWrapper>,
    games: HashMap<isize, GameWrapper>,
    boards: HashMap<isize, Board>,
    commands: HashMap<String, Command>,
    tx_db: Sender<Msg2DbManager>,
    tx_conn: Sender<Msg2ConnManager>
}

impl ProgramState {
    fn new(tx_db: Sender<Msg2DbManager>, tx_conn: Sender<Msg2ConnManager>, commands: HashMap<String, Command>) -> Self {
        Self {
            protocols: Default::default(),
            users_online: Default::default(),
            games: Default::default(),
            boards: Default::default(),
            commands,
            tx_db,
            tx_conn
        }
    }

    fn welcome_screen(&self, link: &ProtocolLink) -> String {
        String::from("NOT MUCH TO SEE YET!")
    }
}

#[derive(Debug)]
pub enum Msg2Lobby {
    NewProtocol(ProtocolLink, oneshot::Sender<ConnectResponse>),
    ProtocolCommand(String, String),
    ProtocolData(String, serde_json::Value)
}

#[derive(Debug)]
pub struct Lobby {
    state: ProgramState,
    pub tx_lobby: Sender<Msg2Lobby>,
    rx_lobby: Receiver<Msg2Lobby>,
    tx_db: Sender<Msg2DbManager>,
    tx_conn_manager: Sender<Msg2ConnManager>,
    running: bool,
    cmd_re: Regex
}

impl Lobby {
    pub fn new(tx_lobby: Sender<Msg2Lobby>, rx_lobby: Receiver<Msg2Lobby>,
        tx_db: Sender<Msg2DbManager>, tx_conn_manager: Sender<Msg2ConnManager>,
        commands: HashMap<String, Command>) -> Self {

        Self {
            state: ProgramState::new(tx_db.clone(), tx_conn_manager.clone(), commands),
            tx_conn_manager,
            rx_lobby,
            tx_lobby,
            tx_db,
            running: true,
            cmd_re: Regex::new(r"(?si)(?P<wholecmd>(?P<prefix>[-.]+)?(?P<cmd>\w+))(?P<switches>(\/\S+)+?)?(?:\s+(?P<args>(?P<lhs>[^=]+)(?:=(?P<rhs>.*))?)?)?").unwrap()
        }
    }

    pub async fn run(&mut self) {
        while self.running {
            if let Some(msg) = self.rx_lobby.recv().await {
                println!("Lobby got a message: {:?}", msg);
                let _ = self.process_lobby_message(msg).await;
            }
        }
    }

    async fn process_lobby_message(&mut self, msg: Msg2Lobby) {
        match msg {
            Msg2ProtocolManager::NewProtocol(mut link, send) => {
                let _ = self.new_protocol(link, send).await;
            },
            Msg2ProtocolManager::ProtocolCommand(conn_id, command) => {
                let _ = self.execute_command(conn_id, command).await;
            },
            Msg2ProtocolManager::ProtocolDisconnected(conn_id) => {
                println!("SESSION {} DISCONNECTED!", conn_id);
                self.protocols.remove(&conn_id);
            }
        }
    }

    async fn new_protocol(&mut self, link: ProtocolLink, send: oneshot::Sender<ConnectResponse>) {
        // There will be some logic here for checking if this IP address should be allowed to connect...
        // For now, just allow it.
        let welcome = self.state.welcome_screen(&link);
        let mut tx = link.tx_protocol.clone();
        self.state.protocols.insert(link.conn_id.clone(), ProtocolWrapper::from(link));
        let _ = send.send(ConnectResponse::Ok).await;
        let _ = tx.send(Msg2MudProtocol::Line(welcome)).await;
    }

    async fn execute_command(&mut self, conn_id: String, command: String) {
        println!("LOBBY GOT COMMAND FROM {}: {}", conn_id, command);

        // letting the above stand for debugging/reference right now.
        if self.state.protocols.contains_key(&conn_id) {
            let prot = self.state.protocols.get().unwrap().clone();

            if self.cmd_re.is_match(&command) {
                // If we match the input regex, then we can do anything!

                let mut captures: HashMap<String, String> = Default::default();
                let caps = self.cmd_re.captures(&command).unwrap();

                for name in ("wholecmd", "prefix", "cmd", "switches", "args", "lhs", "rhs") {
                    let found = caps.name(name);
                    if Some(matched) = found {
                        captures.insert(name, matched.as_str());
                    }
                }

                let whole = captures.get("wholecmd").unwrap().clone();

                if whole.starts_with("-") {
                    // This is for normal commands.

                    if let Some(comm) = self.state.commands.get(&whole) {
                        let use_command = comm.clone();
                    }


                    return;
                } else if whole.starts_with(".") {
                    // This is for channel commands.

                    return;
                } else {
                    // Welp, nothing's gonna match...
                }

            } else {
                // But if we don't, we should go 'Huh? I dont recognize that command. type -help for help'
            }

            if command.starts_with("-") {
                // This is a system command such as logging in, changing a password, or joining a
                // game.

                let mut use_comm: Option<Command> = None;

                for comm in self.state.commands.iter() {
                    if comm.re.is_match(&command) {
                        // This command is a match.
                        use_comm = Some(comm.clone());
                        break;
                    }
                }

                if let Some(cm) = use_comm {
                    let call = cm.func;
                    let _ = call(conn_id, command, cm, &mut self.state).await;
                } else {
                    // What the user typed did not match a command.
                }

                // Do stuff here.
                return;
            }
            if command.starts_with(".") {
                // This is a channel command.

                // Do stuff here.
                return;
            }
        }
    }
}