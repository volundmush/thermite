use thermite_mush::{
    penn::db::core::GameState,
    penn::db::flatfile::{FlatFileReader, FlatLine}
};

use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;

fn main() -> std::io::Result<()> {
    //let mut state = GameState::default();
    //println!("Hello TherMUSH: {:?}", state);

    let mut outdb = File::open("/home/volund/exthird/outdb")?;
    let mut bufreader = BufReader::new(outdb);

    let mut flatdb = FlatFileReader::new(bufreader);

    loop {
        if let Some(res) = flatdb.next() {
            match res {
                Ok(line) => {
                    println!("{:?}", line);
                },
                Err(e) => {
                    eprintln!("{}", e);
                    break;
                }
            }
        } else {
            break
        }
    }

    Ok(())

}
