use std::{
    collections::{HashMap, HashSet}
};

use crate::{
    softcode::typedefs::{DbRef, Timestamp, Money}
};

use legion::Entity;
use lasso::Spur;

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
    pub powers: HashSet<Entity>
}

pub struct ObjAlias {
    pub entity: Entity,
    pub aliases: HashMap<Spur, Spur>
}

pub struct ObjLocks {
    pub locks: HashMap<Entity, LockDef>
}

pub struct Action {
    pub enactor: DbRef,
    pub text: String,
}

pub struct QueueComponent {
    pub me: Entity,
    pub dbref: DbRef,
    pub actions: Vec<Action>
}