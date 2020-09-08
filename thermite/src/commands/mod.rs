pub mod bbs;
pub mod chan;
pub mod game;
pub mod login;
pub mod system;
pub mod user;

use thermite_protocol::ProtocolLink;
use crate::lobby::ProgramState;
use tokio::macros::support::Future;
use std::collections::HashMap;
use std::error::Error;

pub enum HelpCategory {
    User,
    BBS,
    Channel,
    Games,
    System,
}

pub enum CommandAction {
    UserMessage(isize, String)
}

#[derive(Clone, Debug)]
pub struct Command {
    pub name: String,
    pub help_text: Option<String>,
    pub help_category: Option<HelpCategory>,
    pub admin_level: u8,
    pub login_command: bool,
    pub action: fn(&String, &String, &HashMap<String, String>, &mut ProgramState) -> Vec<CommandAction>
}

impl Command {
    fn new(name: String, help_text: Option<String>, help_category: Option<HelpCategory>, admin_level: u8, login_command: bool,
           action: fn(&String, &String, &HashMap<String, String>, &mut ProgramState) -> Vec<CommandAction>) -> Self {
        Self {
            name,
            help_text,
            help_category,
            admin_level,
            login_command,
            action
        }
    }

    pub async fn display_help(&self, conn_id: &String, state: &mut ProgramState) {
        
    }
}