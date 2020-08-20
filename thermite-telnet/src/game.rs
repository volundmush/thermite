use actix::prelude::*;
use crate::entity::{Entity, EntityManager, EntityLocation, EntityName, Kind};

pub struct GameActor {
    pub entity_manager: EntityManager,
}

impl Actor for GameActor {
    type Context = Context<Self>;
}

