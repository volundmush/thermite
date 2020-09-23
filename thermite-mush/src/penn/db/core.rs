use serde_json;
use serde_derive;
use std::collections::{HashSet, HashMap};
use std::fmt::{Display, Formatter};
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;
use std::error::Error;

use super::{
    attributes::{AttributeManager},
    functions::{FunctionManager},
    flags::{FlagManager},
    commands::{CommandManager},
    objects::{ObjManager}
};


#[derive(Debug, Default)]
pub struct GameState {
    pub objects: ObjManager,
    pub flags: FlagManager,
    pub powers: FlagManager,
    pub attributes: AttributeManager,
    pub functions: FunctionManager,
    pub commands: CommandManager,
    pub connections: HashSet<String>
}


#[derive(Debug)]
pub struct DbError {
    data: String
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!( f, "{}", self.data)
    }
}

impl DbError {
    pub fn new(src: &str) -> Self {
        Self {
            data: src.to_string()
        }
    }
}

impl From<&str> for DbError {
    fn from(src: &str) -> Self {
        Self {
            data: src.to_string()
        }
    }
}

impl From<String> for DbError {
    fn from(src: String) -> Self {
        Self {
            data: src
        }
    }
}

impl Error for DbError {

}