use std::io::prelude::*;
use std::fs::File;
use std::error::Error;

use std::{
    io::{Read, BufRead, BufReader}
};

use legion::*;
use thermite_mush::{
    components::{
        props::*
    },
    queries::{
        props::*
    }
};

fn main() -> Result<(), Box<dyn Error>> {

    let mut world = World::default();
    let mut wiz = get_or_create::<FlagMarker>(&mut world, "WIZARD");
    let find = find_name::<FlagMarker>(&mut world, "WIZARD");
    println!("WHAT IS A WIZARD? {:?}", wiz);
    println!("FOUND A WIZARD: {:?}", find);
    let nofind = find_name::<AttributeMarker>(&mut world, "WIZARD");
    println!("FOUND ANYTHING? {:?}", nofind);
    Ok(())

}
