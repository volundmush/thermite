use std::{
    io::{Read, BufRead},
    error::Error,
    collections::{HashMap, HashSet},
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
    FlatLine,
    NodeValue,
    get_idx,
};

#[derive(Debug, Clone)]
pub struct DbNode {
    pub name: String,
    pub value: NodeValue,
    pub children: HashMap<String, DbNode>
}

impl From<FlatLine> for DbNode {
    fn from(src: FlatLine) -> Self {
        Self {
            name: src.name,
            value: src.value,
            children: Default::default(),
        }
    }
}


pub fn load_version(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

}

pub fn load_flag_aliases(data: &[FlatLine]) -> Result<HashMap<String, HashSet<String>>, Box<dyn Error>> {
    let mut out: HashMap<String, HashSet<String>> = Default::default();

    Ok(out)
}

pub fn load_flag_likes(state: &mut GameState, data: &[FlatLine]) -> Result<Vec<Flag>, Box<dyn Error>> {
    let flag_start = get_idx(data, 0, "flagcount", "Could not locate flagcount!")?;
    let alias_start = get_idx(data, 0, "flagaliascount", "Could not locate flagaliascount")?;

    // First we'll load aliases to keep this simple.
    let aliases = load_flag_aliases(&data[alias_start..])?;

    let mut out: Vec<Flag> = Default::default();

    Ok(out)
}

pub fn load_flags(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {
    let flags = load_flag_likes(state, data)?;
}

pub fn load_powers(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {
    let powers = load_flag_likes(state, data)?;
}

pub fn load_attributes(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

}

pub fn load_objects(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

}

pub fn read_v6(mut data: impl Read + BufRead) -> Result<GameState, Box<dyn Error>> {
    let mut out = GameState::default();

    let mut flatlines: Vec<FlatLine> = Default::default();

    let mut reader = FlatFileReader::new(data);

    // Ensure that there were no errors during database parsing. Convert the read FlatLines into a
    // cleaned Vector of them.
    loop {
        if let Some(flat_res) = reader.next() {
            let mut flat = flat_res?;
            flatlines.push(flat);
        } else {
            // we have reached End of Line.
            break;
        }
    }

    let ver_start = get_idx(&flatlines.as_slice(), 0, "+V-", "Cannot locate Version Header")?;
    let flags_start = get_idx(&flatlines.as_slice(), 0, "+FLAGS LIST", "Cannot locate Flags Section")?;
    let powers_start = get_idx(&flatlines.as_slice(), 0, "+POWER LIST", "Cannot locate Power Section")?;
    let attributes_start = get_idx(&flatlines.as_slice(), 0, "+ATTRIBUTES LIST", "Cannot locate Attribute Section")?;
    let objects_start = get_idx(&flatlines.as_slice(), 0, "~", "Cannot locate Objects Section")?;
    let end_of_dump = get_idx(&flatlines.as_slice(), 0, "***END OF DUMP***", "Cannot locate End of Dump")?;

    let ver_end = flags_start - 1;
    let flags_end = powers_start - 1;
    let powers_end = attributes_start - 1;
    let attributes_end = objects_start - 1;
    let objects_end = end_of_dump - 1;

    let _ = load_version(&mut out, &flatlines[ver_start..ver_end])?;
    let _ = load_flags(&mut out, &flatlines[flags_start..flags_end])?;
    let _ = load_powers(&mut out, &flatlines[powers_start..powers_end])?;
    let _ = load_attributes(&mut out, &flatlines[attributes_start..attributes_end])?;
    let _ = load_objects(&mut out, &flatlines[objects_start..objects_end])?;

    Ok(out)
}