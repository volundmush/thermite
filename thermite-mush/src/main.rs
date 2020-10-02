use std::io::prelude::*;
use std::fs::File;
use std::error::Error;

use std::{
    io::{Read, BufRead, BufReader, stdin, stdout, Write}
};

use legion::*;
use thermite_mush::{
    components::{
        props::*
    },
    queries::{
        props::*,
        penn_v6
    }
};

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {

    let mut world = World::default();

    let mut outdb = BufReader::new(File::open("/home/volund/exthird/outdb")?);

    let _ = penn_v6::read_v6(&mut world, outdb)?;

    Ok(())

}
