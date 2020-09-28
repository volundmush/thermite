use std::{
    io::{Read, BufRead},
    error::Error,
    collections::{HashMap, HashSet},
    ops::Range,
    rc::Rc,
    cell::RefCell
};

use super::{
    typedefs::{DbRef, Money, Timestamp},
    core::{DbError, GameState}
};

use super::flatfile::{
    FlatFileReader,
    FlatFileSplitter,
    FlatLine,
    FlatValueNode,
    NodeValue,
};

use thermite_util::{
    text::StringInterner
};

use encoding_rs::*;
use encoding_rs_io::*;


fn load_version(state: &mut GameState, data: &[FlatLine]) -> Result<(), DbError> {

    Ok(())
}

fn _load_flag_alias(mut state: &mut GameState, data: &[FlatLine], type_idx: usize) -> Result<(), DbError> {
    let name_idx = state.props.get_or_create(type_idx, &data[0].text("name", "Improperly formmatted flag alias")?);

    for line in &data[1..] {
        state.props.add_alias(name_idx, &line.text("alias", "improperly formatted flag alias")?);
    }
    Ok(())
}

fn _load_flag_aliases(mut state: &mut GameState, data: &[FlatLine], type_idx: usize) -> Result<(), DbError> {
    let mut name_idx: Vec<usize> = data.iter().enumerate().filter(|(i, x)| x.depth() == 1).map(|(i, x)| i).collect();
    name_idx.push(data.len()+1);
    let mut ranges: Vec<Range<usize>> = name_idx.as_slice().windows(2).map(|x| x[0]..x[1]-1).collect();
    for r in ranges {
        _load_flag_alias(&mut state, &data[r], type_idx)?;
    }
    Ok(())
}

fn _load_flag_single(mut state: &mut GameState, data: &[FlatLine], perm_idx: usize, type_idx: usize) -> Result<usize, DbError> {
    let prop_idx = state.props.get_or_create(type_idx, &data[0].text("name", "Improperly formatted flag data")?);
    state.props.set_letter(prop_idx, &data[1].text("letter", "Improperly formatted flag data")?)?;

    for name in data[2].text("type", "Improperly formatted flag data")?.split_ascii_whitespace() {
        // link the type here!
    }

    for name in data[3].text("perms", "Improperly formatted flag data")?.split_ascii_whitespace() {
        // link flag perms here!
    }

    for name in data[4].text("negate_perms", "improperly formatted flag data")?.split_ascii_whitespace() {
        // link negate perms here!
    }

    Ok((prop_idx))
}

fn _load_flags(mut state: &mut GameState, data: &[FlatLine], perm_idx: usize, type_idx: usize) -> Result<(), DbError> {
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
        let mut flag_idx = _load_flag_single(state,&data[r], perm_idx, type_idx)?;
    }
    _load_flag_aliases(state, &data[alias_idx..], type_idx)?;

    Ok(())
}

fn load_flags(mut state: &mut GameState, data: &[FlatLine]) -> Result<(), DbError> {
    let perm_idx = state.props.get_or_create_type("FLAG_PERM");
    let type_idx = state.props.get_or_create_type("FLAG");
    _load_flags(&mut state, data, perm_idx, type_idx)?;
    Ok(())
}

fn load_powers(mut state: &mut GameState, data: &[FlatLine]) -> Result<(), DbError> {
    let perm_idx = state.props.get_or_create_type("FLAG_PERM");
    let type_idx = state.props.get_or_create_type("POWER");
    _load_flags(&mut state, data, perm_idx, type_idx)?;
    Ok(())
}

fn _load_attr_single(mut state: &mut GameState, data: &[FlatLine], attr_idx: usize, flag_idx: usize) -> Result<usize, DbError> {
    let name = data[0].text("name", "Improperly formatted attribute data")?;
    let name_idx = state.props.get_or_create(attr_idx, &name);

    for name in data[1].text("flags", "improperly formatted attribute data")?.split_ascii_whitespace() {
        // Link attribute flags here!
    }
    let attr = state.props.contents.get_mut(name_idx).unwrap();
    attr.owner = data[2].dbref("creator", "improperly formatted attribute data")?;
    attr.data = data[3].text("data", "improperly formatted attribute data")?;

    Ok(name_idx)
}

fn load_attributes(mut state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {
    let mut alias_idx: usize = 0;
    let mut name_idx: Vec<usize> = Default::default();
    let type_idx = state.props.get_or_create_type("ATTRIBUTE");
    let flag_idx = state.props.get_or_create_type("ATTR_FLAG");

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
        _load_attr_single(&mut state, &data[r], type_idx, flag_idx)?;
    }

    _load_flag_aliases(state, &data[alias_idx..], type_idx)?;

    Ok(())
}

fn _load_obj_lock(mut state: &mut GameState, data: &[FlatLine], lock_idx: usize, flag_idx: usize, obj_idx: usize) -> Result<(), DbError> {
    let name = data[0].text("type", "invalid lock data")?.to_lowercase();
    let creator = data[1].dbref("creator", "invalid lock data")?;
    let flag_data = data[2].text("flags", "invalid lock data")?;
    let key = state.props.lockkeys.get_or_intern(data[4].text("key", "invalid lock data")?);

    let reltype = state.props.reltypes.get_or_intern("flag".to_string());

    for f in flag_data.split_ascii_whitespace() {
        // attach lock flags to lock here...
    }

    Ok(())
}

fn _load_obj_locks(mut state: &mut GameState, data: &[FlatLine], mut obj_data: &mut ObjData) -> Result<(), Box<dyn Error>> {
    for ldata in data[1..].chunks(5) {
        let (name, lock) = _load_obj_lock(&mut state, ldata)?;
        let idx = state.locks.type_interner.get_or_intern(name.as_str());
        obj_data.locks.insert(idx, lock);
    }
    Ok(())
}

fn _load_obj_attrs(mut state: &mut GameState, data: &[FlatLine], mut obj_data: &mut ObjData) -> Result<(), Box<dyn Error>> {

    Ok(())
}

fn _load_obj(mut state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {
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
        let num = info1[0].value_str()[1..].parse::<usize>()?;
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

    let obj_type = state.objects.get_obj_type(type_name).unwrap();

    let mut flags: HashSet<Rc<Flag>> = Default::default();

    for f in info2[4].text("flags", "improperly formatted object data")?.split_ascii_whitespace() {
        if let Some(t) = state.flags.get_flag(f) {
            flags.insert(t);
        } else {
            return Err(DbError::new("Improper flag in object data").into())
        }
    }

    let creation_timestamp = info2[7].num("created", "improperly formatted object data")?;

    let mut obj_data = ObjData {
        name: state.objects.interner.get_or_intern(name.as_str()),
        parent,
        children: Default::default(),
        location,
        contents: Default::default(),
        zoned: Default::default(),
        owner,
        belongings: Default::default(),
        zone,
        money,
        flags,
        modification_timestamp: 0,
        attributes: Default::default(),
        locks: Default::default(),
        connections: Default::default()
    };
    _load_obj_locks(&mut state, lock_data, &mut obj_data)?;
    _load_obj_attrs(&mut state, attr_data, &mut obj_data)?;

    let mut obj = Obj {
        db,
        obj_type,
        creation_timestamp,
        data: RefCell::new(obj_data)
    };

    state.objects.load(obj);
    Ok(())
}

fn load_objects(mut state: &mut GameState, data: &[FlatLine], start: usize, index: &Vec<usize>) -> Result<(), Box<dyn Error>> {
    let mut ranges: Vec<Range<usize>> = index.as_slice().windows(2).map(|x| x[0]..x[1]).collect();

    for r in ranges {
        let obj = _load_obj(&mut state, &data[r])?;
        // Do something with loaded obj here.
    }
    state.objects.load_final();

    Ok(())
}

pub fn read_v6(mut data: impl Read) -> Result<GameState, Box<dyn Error>> {
    // PennMUSH's v6 flatfile is encoded in latin1 which is a subset of WINDOWS_1252.

    let mut out = GameState::default();

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

    let _ = load_version(&mut out, &loaded[ver_start..flags_start-1])?;
    let _ = load_flags(&mut out, &loaded[flags_start..powers_start-1])?;
    let _ = load_powers(&mut out, &loaded[powers_start..attributes_start-1])?;
    let _ = load_attributes(&mut out, &loaded[attributes_start..objects_start-1])?;
    let _ = load_objects(&mut out, &loaded[..], objects_start, &objects_index)?;

    Ok(out)
}