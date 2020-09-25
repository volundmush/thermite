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
    let name = Rc::from(data[0].text("name", "Improperly formmatted flag alias")?);
    let mut aliases: HashSet<Rc<str>> = Default::default();

    for line in &data[1..] {
        aliases.insert(Rc::from(line.text("alias", "improperly formatted flag alias")?));
    }
    Ok((name, aliases))
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
    let name = data[0].text("name", "Improperly formatted flag data")?;
    let letter = data[1].text("letter", "Improperly formatted flag data")?;

    let mut obj_types: HashSet<Rc<ObjType>> = Default::default();

    for name in data[2].text("type", "Improperly formatted flag data")?.split_ascii_whitespace() {
        if let Some(t) = state.objects.get_obj_type(name) {
            obj_types.insert(t);
        } else {
            return Err(DbError::new("Improper type in a flag").into())
        }
    }

    let mut perms: HashSet<Rc<FlagPerm>> = Default::default();

    for name in data[3].text("perms", "Improperly formatted flag data")?.split_ascii_whitespace() {
        if let Some(t) = state.flag_perms.get_flag_perm(name) {
            perms.insert(t);
        } else {
            return Err(DbError::new("Improper perm in a flag").into())
        }
    }

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

    let name = data[0].text("name", "Improperly formatted attribute data")?;

    let mut flags: HashSet<Rc<AttributeFlag>> = Default::default();

    for name in data[1].text("flags", "improperly formatted attribute data")?.split_ascii_whitespace() {
        if let Some(t) = state.attributes.get_attr_flag(name) {
            flags.insert(t);
        } else {
            return Err(DbError::new("Improper attribute flag in attribute").into())
        }
    }

    let creator = data[2].dbref("creator", "improperly formatted attribute data")?;

    let attr_data = data[3].text("data", "improperly formatted attribute data")?;

    Ok(Attribute {
        name: Rc::from(name),
        flags,
        data: attr_data,
        aliases: Default::default(),
        internal: false,
        creator
    })
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

fn _load_obj(state: &GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

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
        info1[0].value_str().parse::<Dbref>()?
    };

    let name = info1[1].text("name", "Improperly formatted object data")?;
    let location = info1[2].dbref("location", "Improperly formatted object data")?;
    let parent = info1[6].dbref("parent", "Improperly formatted object data")?;

    let owner = info2[0].dbref("owner", "Improperly formatted object data")?;
    let zone = info2[1].dbref("zone", "improperly formatted object data")?;
    let pennies = info2[2].num("pennies", "improperly formatted object data")?;

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

    let obj_type = state.objects.get_obj_type(type_name);

    let mut flags: HashSet<Rc<RefCell<Flag>>> = Default::default();

    for f in info2[4].name("flags", "improperly formatted object data")?.split_ascii_whitespace() {
        if let Some(t) = state.attributes.get_attr_flag(name) {
        flags.insert(t);
    } else {
        return Err(DbError::new("Improper attribute flag in attribute").into())
    }
}

    Ok(())
}

fn load_objects(state: &mut GameState, data: &[FlatLine], start: usize, index: &Vec<usize>) -> Result<(), Box<dyn Error>> {
    let mut ranges: Vec<Range<usize>> = index.as_slice().windows(2).map(|x| x[0]..x[1]).collect();

    for r in ranges {
        let obj = _load_obj(&state, &data[r])?;
        // Do something with loaded obj here.
    }

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