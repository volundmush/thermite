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
    FlatFileSplitter,
    FlatLine,
    FlatValueNode,
    NodeValue,
};

use encoding_rs::*;
use encoding_rs_io::*;


fn load_version(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

}

fn load_flags(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

}

fn load_powers(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

}

fn load_attributes(state: &mut GameState, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

}

fn _load_obj(state: &mut GameState, db: Dbref, data: &[FlatLine]) -> Result<(), Box<dyn Error>> {

}

fn load_objects(state: &mut GameState, data: &[FlatLine], start: usize, index: &Vec<usize>) -> Result<(), Box<dyn Error>> {

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