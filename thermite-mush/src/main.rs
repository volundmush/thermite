use thermite_mush::{
    penn::db::core::GameState,
    penn::db::flatfile::{FlatFileReader, FlatFileSplitter, FlatLine},
    penn::db::v6::read_v6,
    penn::mushcode::parser::{
        split_action_list,
        split_argument_list,
        identify_function_squares,
        eval_squares
    }
};

use std::io::prelude::*;
use std::fs::File;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {

    //let mut gamestate = read_v6(File::open("/home/volund/exthird/outdb")?)?;

    let code = r#"@set %#=BOO:This is cool;@select/inline 2=1,{Rawr!},2,{RAAAA\}AWR};@tel me=Place;@assert/inline 1=Do this;@break/inline 2={@pemit %#=rawr!;@pemit %#=rawr2!}"#;

    let idx = split_action_list(code);
    println!("{:?}", idx);

    for r in idx {
        println!("COMMAND: {}", &code[r]);
    }

    let code2 = r#"add(2,3),boo,this\, is a string,what about [rawr]"#;

    let idx = split_argument_list(code2);
    println!("{:?}", idx);

    for r in idx {
        println!("ARGUMENT: {}", &code2[r]);
    }

    let code3 = r#"Hey [this should be parsed] but not this [yes [this]]"#;

    if let Some(idx) = identify_function_squares(code3) {
        for r in idx {
            println!("PARSETHIS: {}", &code3[r]);
        }
    }

    let code4 = r#"add(5,mul(4,power(5,2)))[iter(boo|moo|zam,left(%i0,4)[ljust(%i0,5,c)],|,|)] %# %% bah"#;
    println!("{}", eval_squares(code4));

    Ok(())

}
