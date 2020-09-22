use serde::prelude::*;
use serde_json;
use serde_derive;
use std::collections::{HashSet, HashMap};
use std::fmt::{Display, Formatter};
use std::convert::TryFrom;
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;

pub type Dbref = isize;
pub type Timestamp = usize;
pub type Money = isize;




pub enum FunctionRestriction {
    God,
    Wizard,
    Admin,
    NoGagged,
    NoFixed,
    NoGuest,
    Nobody,
    Logname,
    Logargs,
    Noparse,
    Localize,
    Userfn,
    Nosidefx,
    Deprecated,
    NoPlayer
}

pub enum FunctionAction {
    Builtin(fn() -> ()),
    User(Dbref, usize)
}

pub struct Function {
    pub name: String,
    pub restrictions: HashSet<FunctionRestriction>,
    pub min_args: isize,
    pub max_args: isize,
    pub even_args: bool,
    pub aliases: HashSet<String>
}

#[derive(Default)]
pub struct FunctionManager {
    pub functions: Vec<Function>,
    pub func_index: HashMap<String, usize>
}

pub enum CommandFlag {
    NoParse,
    EqSplit,
    LsArgs,
    RsArgs,
    RsNoParse,
    NoEval
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

#[derive(Default)]
pub struct CommandManager {
    pub commands: Vec<Command>,
    pub name_index: HashMap<String, usize>,
}

pub enum FlagPerm {
    Trusted,
    Royalty,
    Wizard,
    God,
    Dark,
    Mdark,
    Odark
}


pub struct Flag {
    pub name: String,
    pub letter: String,
    pub obj_types: HashSet<usize>,
    pub perms: HashSet<FlagPerm>,
    pub negate_perms: HashSet<FlagPerm>,
    pub aliases: HashSet<String>
}

#[derive(Default)]
pub struct FlagManager {
    pub flags: HashMap<String, Rc<RefCell<Flag>>>,
    pub letter_index: HashMap<char, Rc<RefCell<Flag>>>,
    pub type_index: HashMap<Rc<RefCell<ObjType>>, HashSet<Rc<RefCell<Flag>>>>,
    pub alias_index: HashMap<String, Rc<RefCell<Flag>>>,
}

impl FlagManager {
    pub fn load(&mut self, flags: Vec<Flag>) -> Result<(), DbError> {
        for flag in flags {
            self.add_flag(flag)?;
        }
        Ok(())
    }

    pub fn add_flag(&mut self, flag: Flag) -> Result<(), DbError> {
        let name = flag.name.clone();
        let letter = flag.letter.clone();
        let types = flag.obj_types.clone();
        let aliases = flag.aliases.clone();

        if self.full_index.contains_key(name.as_str()) {
            return Err(DbError::new("A flag with this name or alias already exists."))
        }

        if self.letter_index.contains_key(letter.as_str()) {
            return Err(DbError::new("A flag with this letter already exists."))
        }

        for alias in &aliases {
            if self.full_index.contains_key(alias.as_str()) {
                return Err(DbError::new("A flag with this name or alias already exists."))
            }
        }
        // All verifications have passed - perform the add.
        self.flags.push(flag);
        let idx = self.flags.len();
        self.name_index.insert(name.clone(), idx);
        self.full_index.insert(name.clone(), idx);
        self.letter_index(letter, idx);

        for t in types {
            if !self.type_index.contains_key(&t) {
                self.type_index.insert(t, HashSet::default())
            }
            if let Some(ind) = self.type_index.get_mut(&t) {
                ind.insert(idx);
            }
        }

        Ok(())
    }
}

pub struct LockFlag {
    name: String
}

pub struct LockType {
    name: String,
    custom: bool
}

pub struct Lock {
    pub creator: Dbref,
    pub flags: HashSet<LockFlag>,
    pub key: String
}

pub struct ObjType {
    name: String,
    letter: String,
}

pub struct ObjTypeManager {
    pub objtypes: Vec<ObjType>,
    pub name_idx: HashMap<String, usize>,
    pub letter_idx: HashMap<String, usize>
}

impl TryFrom<&str> for ObjType {
    type Error = DbError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "ROOM" => Ok(Self::Room),
            "EXIT" => Ok(Self::Exit),
            "THING" => Ok(Self::Thing),
            "PLAYER" => Ok(Self::Player),
            _ => Err(DbError::from(format!("Cannot deserialize ObjType {}", value)))
        }
    }
}

pub struct ObjAttr {
    pub index: usize,
    pub value: String,
    pub flags: HashSet<AttributeFlag>,
    pub owner: Dbref
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Obj {
    pub num: Dbref,
    pub name: String,
    pub parent: Dbref,
    pub children: HashSet<Dbref>,
    pub exits: HashSet<Dbref>,
    pub contents: HashSet<Dbref>,
    pub zoned: HashSet<Dbref>,
    pub owner: DbRef,
    pub zone: DbRef,
    pub money: Money,
    pub obj_type: ObjType,
    pub flags: HashSet<Rc<RefCell<Flag>>>,
    pub creation_timestamp: Timestamp,
    pub modification_timestamp: Timestamp,
    pub attributes: HashMap<usize, ObjAttr>,
    pub locks: HashMap<LockType, Lock>,
    pub connections: HashSet<String>
}

impl Default for Obj {
    fn default() -> Self {
        Self {
            num: -1,
            name: "Garbage".to_string(),
            parent: -1,
            children: Default::default(),
            exits: Default::default(),
            contents: Default::default(),
            zoned: Default::default(),
            owner: -1,
            zone: -1,
            money: 0,
            obj_type: ObjType::Garbage,
            creation_timestamp: 0,
            modification_timestamp: 0,
            attributes: Default::default(),
            locks: Default::default(),
            connections: Default::default()
        }
    }
}

impl DbObj {
    pub fn objid(&self) -> String {
        format!("#{}:{}", self.num, self.creation_timestamp)
    }
}

#[derive(Default)]
pub struct ObjManager {
    pub objects: Vec<Obj>,
    pub pmatches: HashMap<String, usize>,
}

#[derive(Default)]
pub struct GameState {
    pub objects: ObjManager,
    pub flags: FlagManager,
    pub powers: FlagManager,
    pub attributes: AttributeManager,
    pub functions: FunctionManager,
    pub commands: CommandManager,
    pub connections: HashSet<String>
}


#[derive(Debug, Display, Error)]
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