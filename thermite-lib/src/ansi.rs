use logos::{Logos, Lexer, Span};

use std::{
    str::FromStr,
    collections::HashMap
};

pub const ESC: &str = "\x1b";

#[derive(Debug, PartialEq)]
pub enum ColorCode {
    Ansi(u8),
    Xterm(u8)
}

//rgybmcwx 
fn parse_ansi_color(src: &str) -> usize {
    match src {
        "x" => {
            30
        },
        "r" => {
            31
        },
        "g" => {
            32
        },
        "y" => {
            33
        },
        "b" => {
            34
        },
        "m" => {
            35
        },
        "c" => {
            36
        },
        "w" => {
            37
        },
        _ => {
            0
        }
    }
}

fn parse_ansi_bg_color(src: &str) -> usize {
    let res = parse_ansi_color(&src.to_lowercase());
    if res > 0 {
        res + 10
    }
    else {
        0
    }
}

fn parse_xterm_color(src: &str) -> usize {

}

fn getstr(lex: &mut Lexer<Token>) -> Option<String> {
    Some(String::from(lex.slice()))
}

fn get_ansi_basic_fg(lex: &mut Lexer<Token>) -> Option<ColorCode> {
    let found = String::from(lex.slice());
    let result = parse_ansi_bg_color(&found[1..].to_lowercase());
    if result > 0 {
        Some(ColorCode::Ansi(result))
    }
    else {
        None
    }
}

fn get_ansi_basic_bg(lex: &mut Lexer<Token>) -> Option<ColorCode> {
    let found = String::from(lex.slice());
    let result = parse_ansi_color(&found[2..].to_lowercase());
    if result > 0 {
        Some(ColorCode::Ansi(result))
    }
    else {
        None
    }
}

fn get_xterm_hibg(lex: &mut Lexer<Token>) -> Option<ColorCode> {

}

fn get_xterm_fg(lex: &mut Lexer<Token>) -> Option<ColorCode> {

}

fn get_xterm_bg(lex: &mut Lexer<Token>) -> Option<ColorCode> {

}

fn get_xgrey_fg(lex: &mut Lexer<Token>) -> Option<ColorCode> {

}

fn get_xgrey_bg(lex: &mut Lexer<Token>) -> Option<ColorCode> {

}


#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[token("||")]
    Pipe,

    #[token("|n")]
    AnsiReset,

    #[token("|h")]
    AnsiHilite,

    #[token("|H")]
    AnsiUnhilite,

    #[token("|/")]
    AnsiCrlf,

    #[token("|-")]
    AnsiTab,

    #[token("|*")]
    AnsiInverse,

    #[token("|^")]
    AnsiBlink,

    #[token("|_")]
    AnsiSpace,

    #[token("|u")]
    AnsiUnderline,

    #[token(" ")]
    Space,

    #[regex("\\|[RGYBMCWX]", get_ansi_basic_fg)]
    AnsiUnFg(ColorCode),

    #[regex("\\|[rgybmcwx]", get_ansi_basic_fg)]
    AnsiHiFg(ColorCode),

    #[regex("\\|\\[[RGYBMCWX]", get_ansi_basic_bg)]
    AnsiUnBg(ColorCode),

    #[regex("\\|\\[[rgybmcwx]", get_xterm_hibg)]
    XtermHiBg(ColorCode),

    #[regex("\\|([0-5])([0-5])([0-5])", get_xterm_fg)]
    XtermFgNum(ColorCode),

    #[regex("\\|\\[([0-5])([0-5])([0-5])", get_xterm_bg)]
    XtermBgNum(ColorCode),

    #[regex("\\|=([a-z])", get_xgrey_fg)]
    XtermGreyFg(ColorCode),

    #[regex("\\|\\[=([a-z])", get_xgrey_bg)]
    XtermGreyBg(ColorCode),

    #[regex("[^\\s\\|]+", getstr)]
    Word(String),

    #[error]
    Error
}

#[derive(Debug)]
pub enum AnsiPiece {
    Word(String),
    Beep,
    Space,
    Crlf,
    Tab
}

// This struct represents a segment of text wrapped by ANSI encoding.
#[derive(Debug)]
pub struct AnsiSpan {
    fg: Option<ColorCode>,
    bg: Option<ColorCode>,
    hilite: bool,
    underline: bool,
    flash: bool,
    tokens: Vec<AnsiPiece>,
    clean_string: String,
    length: usize
}

impl Default for AnsiSpan {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            hilite: false,
            underline: false,
            flash: false,
            tokens: Default::default(),
            clean_string: Default::default(),
            length: 0
        }
    }
}

impl AnsiSpan {
    pub fn child(&self) -> Self {
        Self {
            fg: self.fg,
            bg: self.bg,
            hilite: self.hilite,
            underline: self.underline,
            flash: self.flash,
            tokens: Default::default(),
            clean_string: Default::default(),
            length: 0
        }
    }

    pub fn len(&self) -> usize {
        0
    }
}

pub struct ANSIString {
    raw_string: String,
    spans: Vec<AnsiSpan>,
    length: usize
}

impl FromStr for ANSIString {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_string = String::from(s);
        let out = Self::from(raw_string);
        Ok(out)
    }
}

impl From<String> for ANSIString {
    fn from(src: String) -> Self {
        let clo = src.clone();
        let lexer = Token::lexer(&clo);

        
        let (spans, length) = ANSIString::do_lex(&lexer);

        Self {
            raw_string: src,
            spans,
            length
        }
    }
}

impl ANSIString {
    pub fn new(src: String) -> Self {
        Self::from(src)
    }

    pub fn plain(&self) -> String {
        //return self.clean_string.clone();
    }

    fn do_lex(lex: Lexer<Token>) -> (Vec<AnsiSpan>, usize) {
        let mut spans: Vec<AnsiSpan> = Default::default();
        let mut span = AnsiSpan::default();
        let mut length: usize = 0;
    
        for (tok, span) in lex.spanned() {
            match tok {
                Token::AnsiUnFg(val) => {
    
                },
                Token::AnsiHiFg(val) => {
    
                }
            }
        }
    
        (spans, length)
    }
}