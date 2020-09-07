use crate::lobby::{ProgramState};
use super::Command;
use regex::Regex;
use crate::commands::HelpCategory;
use std::collections::HashMap;


async fn do_help(conn_id: &String, raw_input: &String, matches: &HashMap<String, String>, state: &mut ProgramState) -> Result<(), Box<dyn Error>> {

}


pub fn load_commands(cmds: &mut HashMap<String, Command>) {
    cmds.insert("-help", Command::new(String::from("-help"), None, Some(HelpCategory::System), 0, false, do_help));

}