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
    ProtocolCapabilities,
    ProtocolLink,
    Msg2MudProtocol,
    ConnectResponse,
    Msg2Game
};

use regex::{Regex, Captures};

use crate::{
    db::Msg2DbManager,
    models::{User, Game, Board, Post, PostRead, Channel, ChannelSub, GameMember, MemberStorage},
    commands::{Command}
};

use chrono::NaiveDateTime;
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
    tx_game: Sender<Msg2Game>
}

impl ProgramState {
    fn new(tx_db: Sender<Msg2DbManager>, tx_game: Sender<Msg2Game>, commands: HashMap<String, Command>) -> Self {
        Self {
            protocols: Default::default(),
            users_online: Default::default(),
            games: Default::default(),
            boards: Default::default(),
            commands,
            tx_db,
            tx_game
        }
    }

    fn welcome_screen(&self, link: &ProtocolLink) -> String {
        String::from("NOT MUCH TO SEE YET!")
    }
}

#[derive(Debug)]
pub enum Msg2Lobby {

}

#[derive(Debug)]
pub struct Lobby {
    state: ProgramState,
    pub tx_lobby: Sender<Msg2Lobby>,
    rx_lobby: Receiver<Msg2Lobby>,
    pub tx_game: Sender<Msg2Game>,
    rx_game: Receiver<Msg2Game>,
    tx_db: Sender<Msg2DbManager>,
    running: bool,
    cmd_re: Regex
}

impl Lobby {
    pub fn new(tx_db: Sender<Msg2DbManager>, commands: HashMap<String, Command>) -> Self {

        let (tx_lobby, rx_lobby) = channel(50);
        let (tx_game, rx_game) = channel(50);

        Self {
            state: ProgramState::new(tx_db.clone(), tx_game.clone(), commands),
            rx_lobby,
            tx_lobby,
            tx_game,
            rx_game,
            tx_db,
            running: true,
            cmd_re: Regex::new(r"(?si)(?P<wholecmd>(?P<prefix>[-.]+)?(?P<cmd>\w+))(?P<switches>(\/\S+)+?)?(?:\s+(?P<args>(?P<lhs>[^=]+)(?:=(?P<rhs>.*))?)?)?").unwrap()
        }
    }

    pub async fn run(&mut self) {
        while self.running {
            tokio::select! {
                l_msg = self.rx_lobby.recv() => {
                    if let Some(msg) = l_msg {
                        println!("Lobby got a message: {:?}", msg);
                        let _ = self.process_lobby_message(msg).await;
                    }
                },
                g_msg = self.rx_game.recv() => {
                    if let Some(msg) = g_msg {
                        let _ = self.process_game_message(msg).await;
                    }
                }
            }
        }
    }

    async fn process_game_message(&mut self, msg: Msg2Game) {
        match msg {

        }
    }

    async fn process_lobby_message(&mut self, msg: Msg2Lobby) {
        match msg {

        }
    }

    async fn new_protocol(&mut self, link: ProtocolLink, send: oneshot::Sender<ConnectResponse>) {
        // There will be some logic here for checking if this IP address should be allowed to connect...
        // For now, just allow it.
        let welcome = self.state.welcome_screen(&link);
        let mut tx = link.tx_protocol.clone();
        self.state.protocols.insert(link.conn_id.clone(), ProtocolWrapper::from(link));
        let _ = send.send(ConnectResponse::Ok);
        let _ = tx.send(Msg2MudProtocol::Line(welcome)).await;
    }

    async fn execute_command(&mut self, conn_id: String, command: String) {
        println!("LOBBY GOT COMMAND FROM {}: {}", conn_id, command);

        // letting the above stand for debugging/reference right now.
        if self.state.protocols.contains_key(&conn_id) {
            let prot = self.state.protocols.get(&conn_id).unwrap().clone();

            if self.cmd_re.is_match(&command) {
                // If we match the input regex, then we can do anything!

                let mut captures: HashMap<String, String> = Default::default();
                let caps = self.cmd_re.captures(&command).unwrap();

                for maybe_name in self.cmd_re.capture_names() {
                    if let Some(name) = maybe_name {
                        let found = caps.name(name);
                        if let Some(matched) = found {
                            captures.insert(String::from(name), String::from(matched.as_str()));
                        }
                    }
                }

                let whole = captures.get("wholecmd").unwrap().clone().to_lowercase();

                if whole.starts_with("-") {
                    // This is for normal commands.

                    if let Some(comm) = self.state.commands.get(&whole) {
                        let use_command = comm.clone();
                        let result = (use_command.action)(&conn_id, &command, &captures, &mut self.state).await;
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

            if command.starts_with(".") {
                // This is a channel command.

                // Do stuff here.
                return;
            }
        }
    }
}