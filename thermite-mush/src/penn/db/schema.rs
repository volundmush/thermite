use std::{
    collections::{HashSet, HashMap},
};

use super::{
    typedefs::{Timestamp, DbRef, Money},
};

use generational_arena::Index;
use legion::*;

#[derive(Debug)]
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
#[derive(Debug, Default)]
pub struct Property {
    pub property_type: usize,
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
    pub owner: DbRef,
    pub holders: HashSet<Entity>,
    pub data: String
}

impl Property {
    pub fn name_match(&self, type_id: usize, name_id: usize) -> bool {
        !self.deleted && self.property_type_id == type_id && self.name_id == name_id
    }
}

// many properties have ALIASES. these are always uppercase.
#[derive(Debug, Default)]
pub struct Alias {
    pub property_id: Index,
    pub property_type: usize,
    pub name_id: usize,
    pub upper_id: usize
}

impl Alias {
    pub fn name_match(&self, type_id: usize, name_id: usize) -> bool {
        !self.deleted && self.property_type == type_id && self.name_id == name_id
    }
}

#[derive(Debug)]
pub struct PropertyRelation {
    pub property_id: usize,
    pub relation_type_id: usize,
    pub other_id: usize,
    pub other_type_id: usize
}

// this struct maps an object with a property. this is for things like Flags and Powers.
// IE: either the Object 'has' this thing or it does not. Property Type ID is included for
// better indexing and lookups.
#[derive(Debug)]
pub struct ObjectPropertyRelation {
    pub object_id: Entity,
    pub property_id: Index,
    pub property_type_id: usize,
    pub owner: DbRef,
    pub props: HashSet<Index>,
    pub value: String
}

#[derive(Debug)]
pub struct ObjAlias {
    pub object_id: Entity,
    pub object_type_id: usize,
    pub name_id: usize,
    pub upper_id: usize
}