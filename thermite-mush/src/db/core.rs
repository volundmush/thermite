use serde::prelude::*;
use serde_json;
use serde_derive;
use std::collections::{HashSet, HashMap};

pub type Dbref = isize;
pub type Timestamp = usize;
pub type Money = isize;

pub enum AttributeFlag {
    NoCommand,
    NoInherit,
    NoClone,
    MortalDark,
    Wizard,
    Veiled,
    Nearby,
    Locked,
    Safe,
    Visual,
    Public,
    Debug,
    NoDebug,
    Regexp,
    Case,
    NoSpace,
    NoName,
    AaHear,
    AmHear,
    Prefixmatch,
    Quiet,
    Branch
}

pub struct Attribute {
    pub name: String,
    pub flags: HashSet<AttributeFlag>,
    pub data: String,
    pub aliases: HashSet<String>
}

#[derive(Default)]
pub struct AttributeManager {
    pub attributes: Vec<Attribute>,
    pub name_index: HashMap<String, usize>,
    pub alias_index: HashMap<String, usize>,
    pub holders_index: HashMap<usize, HashSet<Dbref>>
}


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
    pub obj_types: HashSet<ObjType>,
    pub perms: HashSet<FlagPerm>,
    pub negate_perms: HashSet<FlagPerm>,
    pub aliases: HashSet<String>
}

#[derive(Default)]
pub struct FlagManager {
    pub flags: Vec<Flag>,
    pub name_index: HashMap<String, usize>,
    pub letter_index: HashMap<String, usize>,
    pub holder_index: HashMap<usize, HashSet<Dbref>>,
    pub type_index: HashMap<ObjType, HashSet<usize>>,
    pub alias_index: HashMap<String, usize>
}

pub enum LockFlag {
    Visual,
    NoInherit,
    NoClone,
    Wizard,
    Locked
}

pub enum LockType {
    Basic,
    Enter,
    Teleport,
    Use,
    Page,
    Zone,
    Parent,
    Link,
    Open,
    Mail,
    User(String),
    Speech,
    Listen,
    Command,
    Leave,
    Drop,
    Dropin,
    Give,
    From,
    Pay,
    Receive,
    Follow,
    Examine,
    Chzone,
    Forward,
    Filter,
    Infilter,
    Control,
    Dropto,
    Destroy,
    Interact,
    Take,
    Mailforward,
    Chown
}

pub struct Lock {
    pub creator: Dbref,
    pub flags: HashSet<LockFlag>,
    pub key: String
}

pub enum ObjType {
    Garbage,
    Room,
    Exit,
    Thing,
    Player
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