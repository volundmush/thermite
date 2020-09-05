use thermite_ansi::evstring::EvString;
use thermite_ansi::ansi::{AnsiString, AnsiRenderRules};

fn main() {
    let mut disp = EvString::from("THIS |ris very red|c and maybe blue |gor green");
    let mut rules = AnsiRenderRules::default();
    rules.ansi = true;
    rules.xterm256 = true;
    println!("{}", disp.ansi_string.render(&rules, false).text);
}