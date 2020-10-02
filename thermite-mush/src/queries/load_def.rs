use serde_json::Value as JV;

use crate::{
    softcode::typedefs::{DbRef, DbError, Timestamp, Money}
};

use legion::*;

pub fn load_defaults(world: &mut World, data: &JV) -> Result<(), DbError> {
    if let JV::Object(dict) = data {


        if let Some(objtype) = dict.get("obj_type") {
            load_objtype(world, objtype)?;
        } else {
            return Err(DbError::new("invalid json from defaults file: obj_type"))
        }

        if let Some(restrict) = dict.get("restrict") {
            load_restrict(world, restrict)?;
        } else {
            return Err(DbError::new("invalid json from defaults file: restrict"))
        }

        if let Some(attrf) = dict.get("attribute_flag") {
            load_attrflag(world, attrf)?;
        } else {
            return Err(DbError::new("invalid json from defaults file: attribute_flag"))
        }

        if let Some(fperm) = dict.get("flag_perm") {
            load_flagperm(world, fperm)?;
        } else {
            return Err(DbError::new("invalid json from defaults file: flag_perm"))
        }

        if let Some(cmdflag) = dict.get("command_flag") {
            load_cmdflag(world, cmdflag)?;
        } else {
            return Err(DbError::new("invalid json from defaults file: command_flag"))
        }

        if let Some(lock) = dict.get("lock") {
            load_lock(world, lock)?;
        } else {
            return Err(DbError::new("invalid json from defaults file: lock"))
        }

    } else {
        return Err(DbError::new("invalid json from defaults file"))
    }
    Ok(())
}

fn load_objtype(world: &mut World, data: &JV) -> Result<(), DbError> {

    Ok(())
}

fn load_restrict(world: &mut World, data: &JV) -> Result<(), DbError> {

    Ok(())
}

fn load_attrflag(world: &mut World, data: &JV) -> Result<(), DbError> {

    Ok(())
}

fn load_flagperm(world: &mut World, data: &JV) -> Result<(), DbError> {

    Ok(())
}

fn load_cmdflag(world: &mut World, data: &JV) -> Result<(), DbError> {

    Ok(())
}

fn load_lock(world: &mut World, data: &JV) -> Result<(), DbError> {

    Ok(())
}

fn load_flag(world: &mut World, data: &JV) -> Result<(), DbError> {

    Ok(())
}

fn load_power(world: &mut World, data: &JV) -> Result<(), DbError> {

    Ok(())
}

fn load_attribute(world: &mut World, data: &JV) -> Result<(), DbError> {

    Ok(())
}

