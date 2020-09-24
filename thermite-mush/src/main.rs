use thermite_mush::{
    penn::db::core::GameState,
    penn::db::flatfile::{FlatFileReader, FlatFileSplitter, FlatLine},
    penn::db::v6::read_v6,
};

use std::io::prelude::*;
use std::fs::File;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {

    let mut gamestate = read_v6(File::open("/home/volund/exthird/outdb")?)?;

    Ok(())

}
