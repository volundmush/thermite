use serde::prelude::*;
use serde_json;
use serde_derive;
use std::collections::HashSet;

type Dbref = isize;
type Timestamp = usize;
type Money = isize;


#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerData {
    pub home: Dbref,
    pub location: Dbref,
    pub contents: HashSet<Dbref>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomData {
    pub exits: HashSet<Dbref>,
    pub contents: HashSet<Dbref>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExitData {
    pub location: Dbref,
    pub destination: Dbref
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThingData {
    pub home: Dbref,
    pub location: Dbref,
    pub contents: HashSet<Dbref>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DbObjData {
    Player(PlayerData),
    Room(RoomData),
    Exit(ExitData),
    Thing(ThingData),
    Garbage
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbObj {
    pub num: Dbref,
    pub name: String,
    pub parent: Dbref,
    pub children: HashSet<Dbref>,
    pub owner: DbRef,
    pub zone: DbRef,
    pub money: Money,
    pub obj_data: DbObjData,
    pub creation_timestamp: Timestamp,
    pub modification_timestamp: Timestamp,
}

impl Default for DbObj {
    fn default() -> Self {
        Self {
            num: -1,
            name: "Garbage".to_string(),
            parent: -1,
            children: Default::default(),
            owner: -1,
            zone: -1,
            money: 0,
            obj_data: DbObjData::Garbage,
            creation_timestamp: 0,
            modification_timestamp: 0
        }
    }
}

impl DbObj {
    pub fn objid(&self) -> String {
        format!("#{}:{}", self.num, self.creation_timestamp)
    }
}