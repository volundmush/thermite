use std::{
    collections::{HashSet}
};
use legion::Entity;

use crate::{
    softcode::typedefs::DbRef
};

#[derive(Default, Debug)]
pub struct ObjTypeMarker;
#[derive(Default, Debug)]
pub struct RestrictMarker;
#[derive(Default, Debug)]
pub struct AttrFlagMarker;
#[derive(Default, Debug)]
pub struct FlagPermMarker;
#[derive(Default, Debug)]
pub struct LockFlagMarker;
#[derive(Default, Debug)]
pub struct CmdFlagMarker;
#[derive(Default, Debug)]
pub struct FlagMarker;
#[derive(Default, Debug)]
pub struct PowerMarker;
#[derive(Default, Debug)]
pub struct AttributeMarker;


#[derive(Debug)]
pub struct PropName {
    pub entity: Entity,
    pub name: String,
    pub system: bool,
    pub internal: bool
}

impl PropName {
    pub fn new(ent: Entity, name: &str) -> Self {
        Self {
            name: name.trim().to_uppercase(),
            system: false,
            entity: ent,
            internal: false
        }
    }
}

#[derive(Debug)]
pub struct PropAliases {
    pub entity: Entity,
    pub aliases: HashSet<String>
}


#[derive(Debug)]
pub struct PropLetter {
    pub entity: Entity,
    pub letter: Option<char>
}

#[derive(Debug)]
pub struct PropPerms {
    pub perms: HashSet<Entity>,
    pub negate_perms: HashSet<Entity>
}

#[derive(Debug)]
pub struct PropSee {
    pub see_perms: HashSet<Entity>
}

#[derive(Debug)]
pub struct PropRestrictFor {
    pub command: bool,
    pub function: bool
}

#[derive(Debug)]
pub struct PropInherit {
    pub inherits: bool,
    pub tree: bool
}

#[derive(Debug)]
pub struct PropAllowTypes {
    pub allowed: HashSet<Entity>
}

#[derive(Debug)]
pub struct PropAttribute {
    pub attrflags: HashSet<Entity>,
    pub creator: DbRef,
    pub data: String
}