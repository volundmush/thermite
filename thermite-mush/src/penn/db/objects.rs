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

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ObjType {
    pub name: &'static str,
    pub letter: &'static str,
}

#[derive(Debug)]
pub struct ObjAttr {
    pub attr: Rc<RefCell<Attribute>>,
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

#[derive(Debug)]
pub struct ObjManager {
    pub objects: HashMap<usize, Rc<RefCell<Obj>>>,
    pub player_names: HashMap<Rc<str>, Rc<RefCell<Obj>>>,
    // Names are stored this way so that a thousand objects named 'East' will not use a ton of RAM.
    pub names: HashSet<Rc<str>>,
    pub objtypes: HashMap<&'static str, Rc<ObjType>>,
    pub letter_idx: HashMap<&'static str, Rc<ObjType>>
}

impl Default for ObjManager {
    fn default() -> Self {
        let mut manager = Self {
            objects: Default::default(),
            player_names: Default::default(),
            names: Default::default(),
            objtypes: Default::default(),
            letter_idx: Default::default()
        };

        for t in vec![
            ObjType {name: "PLAYER", letter: "P"},
            ObjType {name: "ROOM", letter: "R"},
            ObjType {name: "EXIT", letter: "E"},
            ObjType {name: "THING", letter: "E"}
        ] {
            manager.add_obj_type(t)
        }
        manager
    }
}

impl ObjManager {
    pub fn get_obj_type(&self, name: &str) -> Option<Rc<ObjType>> {
        if let Some(t) = self.objtypes.get(name) {
            Some(t.clone())
        } else if let Some(t) = self.letter_idx.get(name) {
            Some(t.clone())
        } else {
            None
        }
    }

    fn add_obj_type(&mut self, t: ObjType) {
        let name = t.name;
        let letter = t.letter;
        let r = Rc::new(t);
        self.objtypes.insert(name, r.clone());
        self.letter_idx.insert(letter, r);
    }
}