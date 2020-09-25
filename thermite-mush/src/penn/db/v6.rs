use std::{
    io::{Read, BufRead},
    error::Error,
    collections::{HashMap, HashSet},
    ops::Range,
    rc::Rc
};

use super::{
    typedefs::{Dbref, Money, Timestamp},
    core::{DbError, GameState},
    attributes::{Attribute, AttributeFlag, AttributeFlagManager, AttributeManager},
    flags::{Flag, FlagPerm, FlagManager},
    locks::{LockType, LockFlag, Lock},
    objects::{Obj, ObjType, ObjAttr}
};

use super::flatfile::{
    FlatFileReader,
    FlatFileSplitter,
    FlatLine,
    FlatValueNode,
    NodeValue,
};

use encoding_rs::*;
use encoding_rs_io::*;


fn load_version(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

    Ok(())
}

fn _load_flag_alias(state: &GameState, data: &[FlatLine]) -> Result<(Rc<str>, HashSet<Rc<str>>), Box<dyn Error>> {

    let name = {
        if !&data[0].name_str().starts_with("name") {
            return Err(DbError::new("Improperly formmatted flag alias").into())
        }
        data[0].value_str().to_string()
    };

    let mut aliases: HashSet<Rc<str>> = Default::default();

    for line in &data[1..] {
        if !line.name_str().starts_with("alias") {
            return Err(DbError::new("Improperly formmatted flag alias").into())
        }
        aliases.insert(Rc::from(line.value_str().to_string()));
    }

    Ok((Rc::from(name), aliases))
}

fn _load_flag_aliases(state: &GameState, data: &[FlatLine], idx: usize) -> Result<HashMap<Rc<str>, HashSet<Rc<str>>>, Box<dyn Error>> {
    let mut out: HashMap<Rc<str>, HashSet<Rc<str>>> = Default::default();

    let mut name_idx: Vec<usize> = Default::default();

    for (i, line) in data.iter().enumerate() {
        if line.depth() == 1 {
            name_idx.push(i);
        }
    }
    name_idx.push(data.len()+1);
    let mut ranges: Vec<Range<usize>> = name_idx.as_slice().windows(2).map(|x| x[0]..x[1]-1).collect();

    Ok(out)
}

fn _load_flag_single(state: &GameState, data: &[FlatLine]) -> Result<Flag, Box<dyn Error>> {

    let name = {
        if !data[0].name_str().starts_with("name") {
            return Err(DbError::new("Improperly formatted flag data: name").into())
        }
        data[0].value_str().to_string()
    };

    let letter = {
        if !data[1].name_str().starts_with("letter") {
            return Err(DbError::new("Improperly formatted flag data: letter").into())
        }
        data[1].value_str().to_string()
    };

    let obj_types_str = {
        if !data[2].name_str().starts_with("type") {
            return Err(DbError::new("Improperly formmatted flag data: type").into())
        }
        data[2].value_str().to_string()
    };

    let mut obj_types: HashSet<Rc<ObjType>> = Default::default();

    for name in obj_types_str.split_ascii_whitespace() {
        if let Some(t) = state.objects.get_obj_type(name) {
            obj_types.insert(t);
        } else {
            return Err(DbError::new("Improper type in a flag").into())
        }
    }

    let perms_str = {
        if !data[3].name_str().starts_with("perms") {
            return Err(DbError::new("Improperly formmatted flag data: perms").into())
        }
        data[3].value_str().to_string()
    };

    let mut perms: HashSet<Rc<FlagPerm>> = Default::default();

    for name in perms_str.split_ascii_whitespace() {
        if let Some(t) = state.flag_perms.get_flag_perm(name) {
            perms.insert(t);
        } else {
            return Err(DbError::new("Improper perm in a flag").into())
        }
    }

    let negate_perms_str = {
        if !data[4].name_str().starts_with("negate_perms") {
            return Err(DbError::new("Improperly formmatted flag data: negate_perms").into())
        }
        data[4].value_str().to_string()
    };

    let mut negate_perms: HashSet<Rc<FlagPerm>> = Default::default();

    for name in negate_perms_str.split_ascii_whitespace() {
        if let Some(t) = state.flag_perms.get_flag_perm(name) {
            negate_perms.insert(t);
        } else {
            return Err(DbError::new("Improper negate_perm in a flag").into())
        }
    }

    Ok(Flag {
        name: Rc::from(name),
        letter: Rc::from(letter),
        obj_types,
        perms,
        negate_perms,
        aliases: Default::default(),
        holders: Default::default()
    })
}

fn _load_flags(state: &GameState, data: &[FlatLine]) -> Result<Vec<Flag>, Box<dyn Error>> {
    let mut out: Vec<Flag> = Default::default();
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

    let mut aliases = _load_flag_aliases(state, data, alias_idx)?;
    for r in ranges {
        let mut flag = _load_flag_single(state, &data[r])?;
        if let Some(aliaset) = aliases.remove(&flag.name) {
            flag.aliases = aliaset;
        }
        println!("Successfully loaded Flag: {:?}", flag);
    }

    Ok(out)
}

fn load_flags(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {
    state.flags.load(_load_flags(&state, data)?)?;
    Ok(())
}

fn load_powers(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {
    state.powers.load(_load_flags(&state, data)?)?;
    Ok(())
}

fn _load_attr_single(state: &GameState, data: &[FlatLine]) -> Result<Attribute, Box<dyn Error>> {

}

fn load_attributes(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {
    let mut attrs: Vec<Attribute> = Default::default();

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
    let mut ranges: Vec<Range<usize>> = name_idx.as_slice().windows(2).map(|x| x[0]..x[1]).collect();

    let mut aliases = _load_flag_aliases(state, data, alias_idx)?;
    for r in ranges {
        let mut attr = _load_attr_single(state, &data[r])?;
        if let Some(aliaset) = aliases.remove(&attr.name) {
            attr.aliases = aliaset;
        }
        state.attributes.add_attribute(attr);
        println!("Successfully loaded Attribute: {:?}", attr);
    }


    Ok(())
}

fn _load_obj(state: &mut GameState, db: Dbref, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

    Ok(())
}

fn load_objects(state: &mut GameState, data: &[FlatLine], start: usize, index: &Vec<usize>) -> Result<(), Box<dyn Error>> {

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