use std::{
    collections::{HashSet, HashMap}
};

use super::{
    typedefs::{Timestamp, DbRef, Money},
};

// This should store the name that is to be 'displayed' - it is mixed case.
pub struct InternString {
    pub row_id: usize,
    pub text: String
}

// Structs above are only ever added to, because indexing who's using these is
// too much trouble. Memory cost of reclaiming and re-using is silly nothing.
// They are not serialized as a list - instead they are simply represented as JSON in a dump or etc.

// Everything below this -can- be deleted and should be ignored while the game is running
// A serialization and reboot should clean it up.

// property names are not case sensitive.
#[derive(Default)]
pub struct Property {
    pub row_id: usize,
    pub property_type_id: usize,
    pub deleted: bool,
    pub name_id: usize,
    // the perms are LockKeys
    pub see_perms: usize,
    pub set_perms: usize,
    pub reset_perms: usize,

    // system properties cannot be deleted or renamed.
    pub system: bool,
    // hidden properties are never shown to players or admin. they are purely
    // used by the code.
    pub hidden: bool,
    // internal properties are used only by code, but -can- be seen.
    pub internal: bool,
    // some properties are represented by a single character. Some aren't.
    pub letter: Option<char>,

    pub owner: DbRef
}

impl Property {
    pub fn name_match(&self, type_id: usize, name_id: usize) -> bool {
        !self.deleted && self.property_type_id == type_id && self.name_id == name_id
    }
}

// many properties have ALIASES. these are always uppercase.
pub struct Alias {
    pub row_id: usize,
    pub property_id: usize,
    pub property_type_id: usize,
    pub deleted: bool,
    pub name_id: usize,
    pub upper_id: usize
}

impl Alias {
    pub fn name_match(&self, type_id: usize, name_id: usize) -> bool {
        !self.deleted && self.property_type_id == type_id && self.name_id == name_id
    }
}

pub struct PropertyRelation {
    pub row_id: usize,
    pub property_id: usize,
    pub relation_type_id: usize,
    pub deleted: bool,
    pub other_id: usize
}

// An Object's DbRef doesn't necessarily have anything to do with its row ID
pub struct Object {
    pub row_id: usize,
    pub deleted: bool,
    pub dbref: DbRef,
    // obj_type is always a property_id. if deleted, this is to be ignored.
    pub obj_type: usize,
    pub name_id: usize,
    pub upper_id: usize,
    pub creation_timestamp: Timestamp,
    pub modification_timestamp: Timestamp,
    pub money: Money,
    pub parent: DbRef,
    pub zone: DbRef,
    pub owner: DbRef,
    // Destination is used purely by exits.
    pub destination: DbRef
}


// This structure will map DbRefs to the appropriate row_id of Object.
pub struct ObjectMap {
    pub row_id: usize,
    pub deleted: bool,
    pub dbref: DbRef,
    pub object_id: usize,
    pub obj_type: usize,
}

// this struct maps an object with a property. this is for things like Flags and Powers.
// IE: either the Object 'has' this thing or it does not. Property Type ID is included for
// better indexing and lookups.
pub struct ObjectPropertyRelation {
    pub row_id: usize,
    pub deleted: bool,
    pub object_id: usize,
    pub property_id: usize,
    pub property_type_id: usize
}

pub struct ObjectDataRelation {
    pub row_id: usize,
    pub deleted: bool,
    pub object_id: usize,
    pub relation_id: usize,
    pub owner: DbRef,
    pub props: HashSet<usize>,
    pub value: String
}