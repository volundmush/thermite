use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    cell::{RefCell, Ref, RefMut}
};

use super::{
    typedefs::{DbRef, Timestamp, Money},
    core::DbError,
    props::{Property, PropertyData, ObjProperty}
};
use std::hash::{Hash, Hasher};
use std::borrow::Borrow;

use thermite_util::{
    text::StringInterner
};

#[derive(Debug, Default, Clone)]
pub struct ObjReverseIndex {
    pub all: HashSet<Rc<Obj>>,
    pub by_type: HashMap<Rc<Property>, HashSet<Rc<Obj>>>
}

#[derive(Debug)]
pub struct ObjData {
    // the name is actually interned.
    pub name: usize,
    pub parent: DbRef,
    pub children: ObjReverseIndex,
    pub location: DbRef,
    pub contents: ObjReverseIndex,
    pub zone: DbRef,
    pub zoned: ObjReverseIndex,
    pub owner: DbRef,
    pub belongings: ObjReverseIndex,
    pub money: Money,
    pub flags: HashSet<Rc<Property>>,
    pub modification_timestamp: Timestamp,
    pub attributes: HashMap<usize, ObjData>,
    // locks are string-interned to lowercase.
    pub locks: HashMap<usize, ObjData>,
    pub connections: HashSet<String>
}

#[derive(Debug)]
pub struct Obj {
    pub db: DbRef,
    pub obj_type: Rc<Property>,
    pub creation_timestamp: Timestamp,
    pub data: RefCell<ObjData>
}

impl Obj {
    pub fn objid(&self) -> String {
        format!("#{}:{}", self.db, self.creation_timestamp)
    }
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        self.db == other.db
    }
}
impl Eq for Obj {}

impl Hash for Obj {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.db.hash(state)
    }
}


#[derive(Debug)]
pub struct ObjManager {
    pub interner: Rc<RefCell<StringInterner>>,
    pub objects: HashMap<DbRef, Rc<Obj>>,
    // As this stores interned strings, it's not used for determining if a name is in use.
    // Instead, this is more useful for determining for doing a case-insensitive comparison
    // and ensuring that a newly-used name is not in use by ANOTHER player - if you want to rename
    // yourself to the same name of a different case, this can be checked.
    pub player_names: HashMap<usize, Rc<Obj>>,
    pub player_aliases: HashMap<usize, Rc<Obj>>,
    // Names are stored this way so that a thousand objects named 'East' will not use a ton of RAM.
}

impl ObjManager {

    pub fn new(interner: Rc<RefCell<StringInterner>>) -> Self {
        Self {
            interner,
            objects: Default::default(),
            player_names: Default::default(),
            player_aliases: Default::default()
        }
    }

    pub fn load(&mut self, o: Obj) {
        // performs initial loading on an object.
    }

    pub fn load_final(&mut self) {

    }
}