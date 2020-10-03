use std::{
    collections::{HashSet}
};

use super::{
    typedefs::{DbRef, DbError}
};

use legion::*;

#[derive(Debug)]
pub struct LockDef {
    pub flags: HashSet<Entity>,
    pub creator: DbRef,
    pub key: String
}