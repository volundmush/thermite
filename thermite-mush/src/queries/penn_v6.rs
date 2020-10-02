use std::{
    io::{Read, BufRead},
    error::Error,
    collections::{HashMap, HashSet},
    ops::Range,
};

use legion::*;
use legion::storage::Component;

use encoding_rs::*;
use encoding_rs_io::*;

use crate::{
    softcode::typedefs::{Timestamp, Money, DbError, DbRef},
    components::{
        props::*
    },
    db::flatfile::*,
    queries::{
        props,
        obj
    }
};

fn load_version(world: &mut World, data: &[FlatLine]) -> Result<(), DbError> {

    Ok(())
}

fn _load_flag_alias<T: Component + Default>(mut world: &mut World, data: &[FlatLine]) -> Result<(), DbError> {
    let flag_ent = props::get_or_create::<T>(world,&data[0].text("name", "Improperly formmatted flag alias")?.to_uppercase());

    for line in &data[1..] {
        //state.props.add_alias(name_idx, &line.text("alias", "improperly formatted flag alias")?);
    }
    Ok(())
}

fn _load_flag_aliases<T: Component + Default>(mut world: &mut World, data: &[FlatLine]) -> Result<(), DbError> {
    let mut name_idx: Vec<usize> = data.iter().enumerate().filter(|(i, x)| x.depth() == 1).map(|(i, x)| i).collect();
    name_idx.push(data.len()+1);
    let mut ranges: Vec<Range<usize>> = name_idx.as_slice().windows(2).map(|x| x[0]..x[1]-1).collect();
    for r in ranges {
        _load_flag_alias::<T>(&mut world, &data[r])?;
    }
    Ok(())
}

fn _load_flag_single<T: Component + Default>(mut world: &mut World, data: &[FlatLine]) -> Result<Entity, DbError> {
    let flag_ent = props::get_or_create::<T>(world,&data[0].text("name", "Improperly formatted flag data")?.to_uppercase());

    //state.props.set_letter(prop_idx, &data[1].text("letter", "Improperly formatted flag data")?)?;

    for name in data[2].text("type", "Improperly formatted flag data")?.split_ascii_whitespace() {
        let otype_ent = props::get_or_create::<ObjTypeMarker>(world, &name.to_uppercase());
    }

    for name in data[3].text("perms", "Improperly formatted flag data")?.split_ascii_whitespace() {
        let fperm_ent = props::get_or_create::<FlagPermMarker>(world, &name.to_uppercase());
    }

    for name in data[4].text("negate_perms", "improperly formatted flag data")?.split_ascii_whitespace() {
        let fperm_ent = props::get_or_create::<FlagPermMarker>(world, &name.to_uppercase());
    }

    Ok(flag_ent)
}

fn _load_flags<T: Component + Default>(mut world: &mut World, data: &[FlatLine]) -> Result<(), DbError> {
    let mut alias_idx: usize = 0;
    let mut name_idx: Vec<usize> = Default::default();

    for (i, line) in data.iter().enumerate() {
        if line.name_str().starts_with("flagaliascount") {
            alias_idx = i;
            break;
        } else if line.name_str().starts_with("name") {
            name_idx.push(i);
        }
    }
    if alias_idx == 0 {
        return Err(DbError::new("Could not locate flagaliascount").into());
    }
    name_idx.push(alias_idx);
    let mut ranges: Vec<Range<usize>> = name_idx.as_slice().windows(2).map(|x| x[0]..x[1]).collect();

    for r in ranges {
        let mut flag_idx = _load_flag_single::<T>(&mut world,&data[r])?;
    }
    _load_flag_aliases::<T>(world, &data[alias_idx..])?;

    Ok(())
}

fn load_flags(mut world: &mut World, data: &[FlatLine]) -> Result<(), DbError> {
    _load_flags::<FlagMarker>(world, data)?;
    Ok(())
}

fn load_powers(mut world: &mut World, data: &[FlatLine]) -> Result<(), DbError> {
    _load_flags::<PowerMarker>(&mut world, data)?;
    Ok(())
}

fn _load_attr_single(mut world: &mut World, data: &[FlatLine]) -> Result<Entity, DbError> {
    let attr_ent = props::get_or_create::<AttributeMarker>(world, data[0].text("name", "Improperly formatted attribute data")?.as_str());

    for name in data[1].text("flags", "improperly formatted attribute data")?.split_ascii_whitespace() {
        let aflag_ent = props::get_or_create::<AttrFlagMarker>(world, name);
    }
    //attr.owner = data[2].dbref("creator", "improperly formatted attribute data")?;
    //attr.data = data[3].text("data", "improperly formatted attribute data")?;

    Ok(attr_ent)
}

fn load_attributes(mut world: &mut World, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {
    let mut alias_idx: usize = 0;
    let mut name_idx: Vec<usize> = Default::default();

    for (i, line) in data.iter().enumerate() {
        if line.name_str().starts_with("attraliascount") {
            alias_idx = i;
            break;
        } else if line.name_str().starts_with("name") {
            name_idx.push(i);
        }
    }
    if alias_idx == 0 {
        return Err(DbError::new("Could not locate attraliascount").into());
    }
    name_idx.push(alias_idx);

    for r in name_idx.as_slice().windows(2).map(|x| x[0]..x[1]) {
        _load_attr_single(world, &data[r])?;
    }

    _load_flag_aliases::<AttributeMarker>(world, &data[alias_idx..])?;

    Ok(())
}

fn _load_obj_lock(mut world: &mut World, data: &[FlatLine], obj: Entity) -> Result<(), DbError> {
    //let name = data[0].text("type", "invalid lock data")?.to_lowercase();
    //let creator = data[1].dbref("creator", "invalid lock data")?;
    //let flag_data = data[2].text("flags", "invalid lock data")?;
    //let key = state.props.lockkeys.get_or_intern(data[4].text("key", "invalid lock data")?);

    //let reltype = state.props.reltypes.get_or_intern("flag".to_string());

    //for f in flag_data.split_ascii_whitespace() {
        // attach lock flags to lock here...
    //}

    Ok(())
}

fn _load_obj_locks(mut world: &mut World, data: &[FlatLine], obj: Entity) -> Result<(), DbError> {
    //for ldata in data[1..].chunks(5) {
    //    let (name, lock) = _load_obj_lock(&mut state, ldata)?;
    //    let idx = state.locks.type_interner.get_or_intern(name.as_str());
    //    obj_data.locks.insert(idx, lock);
    //}
    Ok(())
}

fn _load_obj_attrs(mut world: &mut World, data: &[FlatLine], obj: Entity) -> Result<(), DbError> {

    Ok(())
}

fn _load_obj(mut world: &mut World, data: &[FlatLine]) -> Result<(), DbError> {
    let mut lock_idx: usize = 0;
    let mut attr_idx: usize = 0;
    let mut owner_idx: usize = 0;

    for (i, line) in data.iter().enumerate() {
        if line.name_str().starts_with("lockcount") {
            lock_idx = i;
        } else if line.name_str().starts_with("owner") {
            owner_idx = i;
        } else if line.name_str().starts_with("attrcount") {
            attr_idx = i;
        }
        if lock_idx > 0 && attr_idx > 0 && owner_idx > 0 {
            break;
        }
    }

    if !(lock_idx > 0 && attr_idx > 0 && owner_idx > 0) {
        return Err(DbError::new("Cannot index Object").into())
    }

    let info1 = &data[..lock_idx];
    let lock_data = &data[lock_idx..owner_idx];
    let info2 = &data[owner_idx..attr_idx];
    let attr_data = &data[attr_idx..];
    
    let db = {
        if !info1[0].value_str().starts_with('!') {
            return Err(DbError::new("Invalid Object Data: dbref").into())
        }
        let num = match info1[0].value_str()[1..].parse::<usize>() {
            Ok(res) => res,
            Err(e) => return Err(DbError::new("could not convert number"))
        };
        DbRef::Num(num)
    };

    let name = info1[1].text("name", "Improperly formatted object data")?;
    let location = info1[2].dbref("location", "Improperly formatted object data")?;
    let parent = info1[6].dbref("parent", "Improperly formatted object data")?;

    let owner = info2[0].dbref("owner", "Improperly formatted object data")?;
    let zone = info2[1].dbref("zone", "improperly formatted object data")?;
    let money = info2[2].num("pennies", "improperly formatted object data")?;

    let type_name = {
        match info2[3].num("type", "improperly formatted object data")? {
            8 => "PLAYER",
            1 => "ROOM",
            4 => "EXIT",
            2 => "THING",
            _ => {
                return Err(DbError::new("improper object type").into())
            }
        }
    };

    let obj_type = props::get_or_create::<ObjTypeMarker>(world, type_name);

    for f in info2[4].text("flags", "improperly formatted object data")?.split_ascii_whitespace() {
        if let Some(t) = props::find_name::<FlagMarker>(world, f) {
            //flags.insert(t);
        } else {
            return Err(DbError::new("Improper flag in object data").into())
        }
    }

    let creation_timestamp = info2[7].num("created", "improperly formatted object data")?;

    Ok(())
}

fn load_objects(mut world: &mut World, data: &[FlatLine], start: usize, index: &Vec<usize>) -> Result<(), DbError> {
    let mut ranges = index.as_slice().windows(2).map(|x| x[0]..x[1]).collect::<Vec<_>>();

    for r in ranges {
        let obj = _load_obj(world, &data[r])?;
        // Do something with loaded obj here.
    }


    Ok(())
}

pub fn read_v6(world: &mut World, mut data: impl Read) -> Result<(), Box<dyn Error>> {
    // PennMUSH's v6 flatfile is encoded in latin1 which is a subset of WINDOWS_1252.

    let mut decoder = DecodeReaderBytesBuilder::new().encoding(Some(encoding_rs::WINDOWS_1252)).build(data);
    let mut splitter = FlatFileSplitter::new(decoder);
    let mut outdb = FlatFileReader::new(splitter);

    let mut loaded: Vec<FlatLine> = Default::default();

    let mut ver_start: usize = 0;
    let mut flags_start: usize = 0;
    let mut powers_start: usize = 0;
    let mut attributes_start: usize = 0;
    let mut objects_start: usize = 0;
    let mut objects_index: Vec<usize> = Default::default();

    let mut end_of_dump: usize = 0;

    for (i, res) in outdb.enumerate() {
        match res {
            Ok(data) => {
                match &data {
                    FlatLine::Header(txt) => {
                        if txt.starts_with("+V-") {
                            ver_start = i;
                        } else if txt.starts_with("+FLAGS") {
                            flags_start = i;
                        } else if txt.starts_with("+POWER") {
                            powers_start = i;
                        } else if txt.starts_with("+ATTRIBUTE") {
                            attributes_start = i;
                        } else if txt.starts_with("~") {
                            objects_start = i;
                        } else if txt.starts_with("***END OF DUMP") {
                            end_of_dump = i;
                        } else if txt.starts_with("!") {
                            objects_index.push(i);
                        }
                    },
                    _ => {}
                };
                loaded.push(data);
            },
            Err(e) => {
                return Err(e.into())
            }
        };
    }
    objects_index.push(end_of_dump);

    let _ = load_version(world, &loaded[ver_start..flags_start-1])?;
    let _ = load_flags(world, &loaded[flags_start..powers_start-1])?;
    let _ = load_powers(world, &loaded[powers_start..attributes_start-1])?;
    let _ = load_attributes(world, &loaded[attributes_start..objects_start-1])?;
    let _ = load_objects(world, &loaded[..], objects_start, &objects_index)?;

    Ok(())
}