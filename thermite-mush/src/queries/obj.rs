use crate::{
    softcode::typedefs::{DbRef, DbError},
    queries::props,
    components::{
        props as pcom,
        obj::*
    },
    softcode::{
        managers::DbRefManager
    }
};

use legion::*;

pub fn valid_create(world: &mut World, dbman: &mut DbRefManager, db: DbRef, name: &str, obj_type: Entity) -> Result<DbRef, DbError> {
    // Performs the deepest level of object creation - name conflict checking, type checking, and DbRef availability.
    // if DbRef is DbRef::None, will automatically choose a DbRef.
    // However, nothing is written to database / game state. This merely validates.

    // gotta do name validation here depending on the obj_type

    // Even if names check out, the given DbRef must be figured out or chosen. If so, this is a
    // valid combination...
    Ok(dbman.create(db)?)
}

pub fn delete(world: &mut World, dbman: &mut DbRefManager, db: DbRef) -> Result<(), DbError> {
    if let Some(idx) = dbman.delete(&db) {

        Ok(())
    } else {
        return Err(DbError::new("object does not exist"))
    }
}