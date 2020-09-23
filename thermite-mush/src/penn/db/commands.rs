use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use super::typedefs::Dbref;

#[derive(Debug)]
pub struct CommandFlag {
    name: &'static str
}

#[derive(Debug)]
pub enum CommandAction {
    Builtin(fn() -> ()),
    NotImplemented
}

#[derive(Debug)]
pub struct Command {
    pub name: Rc<str>,
    pub flags: HashSet<Rc<CommandFlag>>,
    pub lock: Rc<str>,
    pub restrict_error: Option<Rc<str>>,
    pub action: CommandAction,
    pub hook_ignore: Option<(Dbref, String)>,
    pub hook_override: Option<(Dbref, String, bool)>,
    pub hook_before: Option<(Dbref, String)>,
    pub hook_after: Option<(Dbref, String)>,
    pub hook_extend: Option<(Dbref, String, bool)>
}

#[derive(Debug)]
pub struct CommandManager {
    pub commands: HashMap<Rc<str>, Rc<Command>>,
    pub flags: HashMap<&'static str, Rc<CommandFlag>>,
}

impl CommandManager {
    fn add_flag(&mut self, flag: CommandFlag) {

    }
}

impl Default for CommandManager {
    fn default() -> Self {
        let mut manager = Self {
            commands: Default::default(),
            flags: Default::default(),
        };

        manager.add_flag(CommandFlag {name: "noparse"});
        manager.add_flag(CommandFlag {name: "eqsplit"});
        manager.add_flag(CommandFlag {name: "lsargs"});
        manager.add_flag(CommandFlag {name: "rsargs"});
        manager.add_flag(CommandFlag {name: "rsnoparse"});

        manager
    }
}
