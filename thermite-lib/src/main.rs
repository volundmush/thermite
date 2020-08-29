use thermite_lib::evstring::{EvString};
use thermite_lib::ansi::strip_ansi;
use std::str::FromStr;

fn main() {
    println!("Testing ANSIString!");
    let mut astr = EvString::from_str("|rTESTING!|n |=aIS |314THIS |mWORKING? |[cHIYA|n").unwrap();
    println!("Stripped form: {}", astr.plain);
    println!("length: {}", astr.width());
    println!("Colored form: {}", astr.render(true, true));
    println!("Stripped: {}", strip_ansi(&astr.render(true, true)));
}
