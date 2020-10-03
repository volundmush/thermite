use std::{
    collections::{HashMap, HashSet}
};

use crate::{
    softcode::{
        typedefs::{DbRef, Timestamp, Money},
        locks::{LockDef}
    }
};

use legion::Entity;
use lasso::Spur;

#[derive(Debug)]
pub struct Obj {
    pub entity: Entity,
    pub dbref: DbRef,
    pub obj_type: Entity,
    pub name: Spur,
    pub uname: Spur,
    pub creation_timestamp: Timestamp,
    pub modification_timestamp: Timestamp,
    pub money: Money,
    pub flags: HashSet<Entity>,
    pub powers: HashSet<Entity>,
    pub owner: DbRef,
    pub destination: DbRef,
    pub location: DbRef,
    pub zone: DbRef,
    pub parent: DbRef,
    pub memberships: HashSet<DbRef>
}

#[derive(Debug)]
pub struct ObjReverse {
    pub entity: Entity,
    pub belongings: HashSet<Entity>,
    pub entrances: HashSet<Entity>,
    pub contents: HashSet<Entity>,
    pub zoned: HashSet<Entity>,
    pub members: HashSet<Entity>,
    pub children: HashSet<Entity>

}

#[derive(Debug)]
pub struct ObjAlias {
    pub entity: Entity,
    pub aliases: HashMap<Spur, Spur>
}

#[derive(Debug)]
pub struct ObjLocks {
    pub locks: HashMap<Entity, LockDef>
}

#[derive(Debug)]
pub struct AttrVal {
    pub owner: DbRef,
    pub flags: HashSet<Entity>,
    pub value: String
}

#[derive(Debug)]
pub struct ObjAttrs {
    pub attrs: HashMap<Entity, AttrVal>
}

#[derive(Debug)]
pub struct Action {
    pub enactor: DbRef,
    pub text: String,
}

#[derive(Debug)]
pub struct QueueComponent {
    pub me: Entity,
    pub dbref: DbRef,
    pub actions: Vec<Action>
}
