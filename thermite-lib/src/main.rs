use thermite_lib::ansi::{ANSIString, Token};
use std::str::FromStr;

fn main() {
    println!("Testing ANSIString!");
    let mut astr = ANSIString::from_str("|rTESTING!|n IS|X |554THIS WORKING? HIYA|n").unwrap();
    println!("Stripped form: {}", astr.plain());
}