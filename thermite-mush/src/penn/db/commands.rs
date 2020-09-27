use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use super::{
    typedefs::DbRef,
    props::{Property, PropertyData, PropertyManager}
};

#[derive(Debug, Clone)]
pub enum CommandAction {
    Builtin(fn() -> ()),
    User
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Hook {
    pub db: DbRef,
    pub attr: usize,
    pub inline: bool
}

#[derive(Debug, Clone)]
pub struct Command {
    pub action: CommandAction,
    pub hook_ignore: Option<Hook>,
    pub hook_override: Option<Hook>,
    pub hook_before: Option<Hook>,
    pub hook_after: Option<Hook>,
    pub hook_extend: Option<Hook>
}

#[derive(Debug, Default)]
pub struct CommandManager {
    pub internal_manager: PropertyManager,
    pub commands: HashMap<usize, Command>,

}