use std::collections::{HashSet, HashMap};
use std::rc::Rc;

pub enum CommandFlag2 {
    NoParse,
    EqSplit,
    LsArgs,
    RsArgs,
    RsNoParse,
    NoEval
}

pub struct CommandFlag {
    name: &'static str
}

pub enum CommandAction {
    Builtin(fn() -> ()),
    NotImplemented
}

pub enum CommandHook {
    Ignore,
    Override(bool),
    Before,
    After,
    Extend(bool)
}

pub struct Command {
    pub name: String,
    pub flags: HashSet<CommandFlag>,
    pub lock: String,
    pub restrict_error: Option<String>,
    pub action: CommandAction,
    pub hooks: HashMap<CommandHook, (Dbref, usize)>
}


pub struct CommandManager {
    pub commands: Vec<Command>,
    pub flags: HashMap<&'static str, Rc<CommandFlag>>
    pub name_index: HashMap<String, usize>,
}

impl Default for CommandManager {
    
}