use legion::*;
use std::{
    collections::{HashSet, HashMap},
    convert::{TryFrom}
};

use super::{
    typedefs::{DbRef, Money, Timestamp, ObjType},
    core::{DbError}
};

use specs::world::Index;
use generational_arena::{Arena, Index};
use lasso::{Rodeo, Spur};

#[derive(Default)]
pub struct ObjRelation {
    pub obj: Option<DbRef>,
    pub reverse: HashSet<DbRef>
}


#[derive(Default)]
pub struct RelationHolder {
    pub parent: ObjRelation,
    pub zone: ObjRelation,
    pub destination: ObjRelation,
    pub owner: ObjRelation,
    pub gate: ObjRelation
}

pub struct ObjectComponent {
    pub me: Entity,
    pub dbref: DbRef,
    pub obj_type: Index,
    pub name_id: usize,
    pub upper_id: usize,
    pub creation_timestamp: Timestamp,
    pub modification_timestamp: Timestamp,
    pub money: Money,
    pub rel: RelationHolder
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