use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    cell::{RefCell, Ref, RefMut}
};

use super::{
    attributes::{Attribute, AttributeFlag},
    typedefs::{Dbref, Timestamp, Money},
    core::DbError,
    flags::{Flag},
    locks::{Lock, LockType}
};

#[derive(Debug)]
pub struct ObjType {
    pub name: &'static str,
    pub letter: &'static str,
}

#[derive(Debug)]
pub struct ObjTypeManager {
    pub objtypes: HashMap<&'static str, Rc<ObjType>>,
    pub letter_idx: HashMap<&'static str, Rc<ObjType>>
}

#[derive(Debug)]
pub struct ObjAttr {
    pub index: usize,
    pub value: String,
    pub flags: HashSet<Rc<AttributeFlag>>,
    pub owner: Dbref
}

#[derive(Clone, Debug)]
pub struct Obj {
    pub num: Dbref,
    pub name: Rc<str>,
    pub parent: Dbref,
    pub children: HashSet<Dbref>,
    pub exits: HashSet<Dbref>,
    pub contents: HashSet<Dbref>,
    pub zoned: HashSet<Dbref>,
    pub owner: Dbref,
    pub zone: Dbref,
    pub money: Money,
    pub obj_type: Rc<ObjType>,
    pub flags: HashSet<Rc<RefCell<Flag>>>,
    pub creation_timestamp: Timestamp,
    pub modification_timestamp: Timestamp,
    pub attributes: HashMap<Rc<Attribute>, Rc<RefCell<ObjAttr>>>,
    pub locks: HashMap<Rc<LockType>, Rc<RefCell<Lock>>>,
    pub user_locks: HashMap<String, Rc<RefCell<Lock>>>,
    pub connections: HashSet<String>
}

impl Obj {
    pub fn objid(&self) -> String {
        format!("#{}:{}", self.num, self.creation_timestamp)
    }
}

#[derive(Default, Debug)]
pub struct ObjManager {
    pub objects: HashMap<usize, Rc<RefCell<Obj>>>,
    pub player_names: HashMap<Rc<str>, Rc<RefCell<Obj>>>,
    // Names are stored this way so that a thousand objects named 'East' will not use a ton of RAM.
    pub names: HashSet<Rc<str>>
}