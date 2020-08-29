use thermite_lib::ansi::{ANSIString, Token};
use std::str::FromStr;

fn main() {
    println!("Testing ANSIString!");
    let mut astr = ANSIString::from_str("|rTESTING!|n |=aIS |314THIS |mWORKING? |[cHIYA|n").unwrap();
    println!("Stripped form: {}", astr.plain);
    println!("length: {}", astr.width());
    println!("Colored form: {}", astr.render(true, true));
}
