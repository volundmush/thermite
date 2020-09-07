pub mod bbs;
pub mod chan;
pub mod game;
pub mod system;
pub mod user;

use crate::protocol::ProtocolLink;
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

#[derive(Clone, Debug)]
pub struct Command {
    pub name: String,
    pub help_text: String,
    pub help_category: HelpCategory,
    pub admin_level: u8,
    pub login_command: bool,
    pub action: fn(&String, &String, &HashMap<String, String>, &mut ProgramState) -> Box<dyn Future<Output = Result<(), Box<dyn Error>>>>
}

impl Command {
    fn new(name: String, help_text: String, help_category: HelpCategory, admin_level: u8, login_command: bool,
           action: fn(&String, &String, &HashMap<String, String>, &mut ProgramState) -> Box<dyn Future<Output = Result<(), Box<dyn Error>>>>) -> Self {
        Self {
            name,
            help_text,
            help_category,
            admin_level,
            login_command,
            action
        }
    }
}