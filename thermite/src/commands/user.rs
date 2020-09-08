use crate::lobby::{ProgramState};
use super::Command;
use regex::Regex;
use crate::commands::HelpCategory;
use std::{
    collections::HashMap,
    error::Error
};


async fn do_user_create(conn_id: &String, raw_input: &String, matches: &HashMap<String, String>, state: &mut ProgramState) -> Result<(), Box<dyn Error>> {

}
