use super::{
    typedefs::Dbref,
    core::DbError
};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc
};

#[derive(Debug)]
pub struct LockFlag {
    pub name: &'static str,
    pub letter: &'static str,
    pub set_perm: &'static str,
    pub reset_perm: &'static str
}

#[derive(Debug)]
pub struct LockType {
    pub name: &'static str,
}

// This is meant to be stored as a HashMap<Rc<LockType>, Rc<RefCell<Lock>>>
// Alternatively, for user locks, a HashMap<Rc<str>, Rc<RefCell<Lock>> could work.
#[derive(Debug)]
pub struct Lock {
    pub creator: Dbref,
    pub flags: HashSet<Rc<LockFlag>>,
    pub key: String
}

#[derive(Debug)]
pub struct LockManager {
    pub locktypes: HashMap<&'static str, Rc<LockType>>,
    pub flags: HashMap<&'static str, Rc<LockFlag>>
}

impl LockManager {
    pub fn add_locktype(&mut self, locktype: LockType) {
        self.locktypes.insert(locktype.name, Rc::new(locktype));
    }

    pub fn add_lockflag(&mut self, lockflag: LockFlag) {
        self.flags.insert(lockflag.name, Rc::new(lockflag));
    }
}

impl Default for LockManager {
    fn default() -> Self {
        let mut manager = Self {
            locktypes: Default::default(),
            flags: Default::default()
        };

        // User: lock is handled in a different way.
        for name in ["basic", "enter", "teleport", "use", "page", "zone", "parent",
        "link", "open", "mail", "speech", "listen", "command", "leave", "drop", "dropin",
        "give", "from", "pay", "receive", "follow", "examine", "chzone", "forward", "filter",
        "infilter", "control", "dropto", "destroy", "interact", "take", "mailforward", "chown"].iter() {
            manager.add_locktype(LockType {name})
        }

        manager.add_lockflag(LockFlag {name: "visual", letter: "v", set_perm: "#TRUE", reset_perm: "#TRUE"});
        manager.add_lockflag(LockFlag {name: "no_inherit", letter: "i", set_perm: "#TRUE", reset_perm: "#TRUE"});
        manager.add_lockflag(LockFlag {name: "no_clone", letter: "c", set_perm: "#TRUE", reset_perm: "#TRUE"});
        manager.add_lockflag(LockFlag {name: "wizard", letter: "w", set_perm: "FLAG^WIZARD", reset_perm: "FLAG^WIZARD"});
        manager.add_lockflag(LockFlag {name: "locked", letter: "+", set_perm: "#TRUE", reset_perm: "#TRUE"});

        manager
    }
}