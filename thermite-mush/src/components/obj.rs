use crate::{
    softcode::typedefs::{DbRef, Timestamp, Money}
};

use legion::Entity;

pub struct ObjectComponent {
    pub me: Entity,
    pub dbref: DbRef,
    pub obj_type: Entity,
    pub creation_timestamp: Timestamp,
    pub modification_timestamp: Timestamp,
    pub money: Money
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