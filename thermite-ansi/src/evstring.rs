use crate::ansi::{
    ColorCode,
    AnsiToken,
    AnsiString,
    AnsiStyle,
    strip_ansi
};

use logos::{Logos, Lexer, Span, Source};


use std::{
    str::FromStr,
    collections::HashMap
};
use regex::internal::Input;

//rgybmcwx 
fn parse_ansi_color(src: &str) -> u8 {
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

fn parse_ansi_bg_color(src: &str) -> u8 {
    let res = parse_ansi_color(&src.to_lowercase());
    if res > 0 {
        res + 10
    }
    else {
        0
    }
}

fn parse_xterm_color(src: &str) -> u8 {
    // This must be fed a string that is 3 digits, 0 to 5.
    if src.len() != 3 {
        return 7;
    }
    let chars = src.as_bytes();
    let red = chars[0] - 48;
    let green = chars[1] - 48;
    let blue = chars[1] - 48;
    16 + (red * 36) + (green * 6) + blue
}

fn getstr(lex: &mut Lexer<Token>) -> Option<String> {
    Some(String::from(lex.slice()))
}

fn get_space_count(lex: &mut Lexer<Token>) -> Option<usize> {
    // Spaces are always a single byte so this is OK.
    Some(String::from(lex.slice()).len())
}

fn get_ansi_basic_fg(lex: &mut Lexer<Token>) -> Option<ColorCode> {
    let found = String::from(lex.slice());
    let result = parse_ansi_color(&found[1..].to_lowercase());
    if result > 0 {
        Some(ColorCode::Ansi(result))
    }
    else {
        None
    }
}

fn get_ansi_basic_bg(lex: &mut Lexer<Token>) -> Option<ColorCode> {
    let found = String::from(lex.slice());
    let result = parse_ansi_bg_color(&found[2..].to_lowercase());
    if result > 0 {
        Some(ColorCode::Ansi(result))
    }
    else {
        None
    }
}

fn get_xterm_fg(lex: &mut Lexer<Token>) -> Option<ColorCode> {
    let found = String::from(lex.slice());
    let result = parse_xterm_color(&found[1..]);
    Some(ColorCode::Xterm(result))
}

fn get_xterm_bg(lex: &mut Lexer<Token>) -> Option<ColorCode> {
    let found = String::from(lex.slice());
    let result = parse_xterm_color(&found[2..]);
    Some(ColorCode::Xterm(result))
}

fn parse_xterm_az(src: &str) -> u8 {
    // a is 97, z is 122. xterm greyscale starts at 231.
    src.as_bytes()[0] + 134
}

fn get_xgrey_fg(lex: &mut Lexer<Token>) -> Option<ColorCode> {
    let found = String::from(lex.slice());
    let result = parse_xterm_az(&found[1..]);
    Some(ColorCode::Xterm(result))
}

fn get_xgrey_bg(lex: &mut Lexer<Token>) -> Option<ColorCode> {
    let found = String::from(lex.slice());
    let result = parse_xterm_az(&found[2..]);
    Some(ColorCode::Xterm(result))
}


#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[token("|")]
    #[token("||")]
    Pipe,

    #[token("|n")]
    Reset,

    #[token("|h")]
    Hilite,

    #[token("\r\n")]
    #[token("|/")]
    Crlf,

    #[token("\t")]
    #[token("|-")]
    Tab,

    #[token("|*")]
    Inverse,

    #[token("|^")]
    Blink,

    #[token("|i")]
    Italic,

    #[token("|s")]
    Strikethrough,

    #[token("|u")]
    Underline,

    #[token("|_")]
    Space,

    #[regex(" +")]
    Spaces,

    #[regex("\\|[rgybmcwx]", get_ansi_basic_fg)]
    AnsiFg(ColorCode),

    #[regex("\\|\\[[rgybmcwx]", get_ansi_basic_bg)]
    AnsiBg(ColorCode),

    #[regex("\\|=([a-z])", get_xgrey_fg)]
    #[regex("\\|([0-5])([0-5])([0-5])", get_xterm_fg)]
    XtermFg(ColorCode),

    #[regex("\\|\\[=([a-z])", get_xgrey_bg)]
    #[regex("\\|\\[([0-5])([0-5])([0-5])", get_xterm_bg)]
    XtermBg(ColorCode),

    #[regex("[^\\s\\|]+")]
    Word,

    #[error]
    Error
}

pub fn parse_evstring(raw: &str) -> Vec<AnsiToken> {
    // This parses an EvString sequence and generates an ANSI-coded string for use with AnsiString.
    let mut out_vec: Vec<AnsiToken> = Vec::default();

    let mut current_style = AnsiStyle::new();
    let mut ansi_change = false;

    for (tok, span) in Token::lexer(raw).spanned() {
        let mut new_style = current_style.clone();

        match tok {
            // These tokens can add text, so are treated differently from the ANSI state changers.
            Token::Pipe | Token::Crlf | Token::Tab | Token::Space | Token::Spaces | Token::Word => {
                // if ANSI state changed, we must flush the change.
                if ansi_change {
                    out_vec.push(AnsiToken::Ansi(new_style.clone()));
                    ansi_change = false;
                }
                match tok {
                    Token::Pipe => out_vec.push(AnsiToken::Text(String::from("|"))),
                    Token::Crlf => out_vec.push(AnsiToken::Newline),
                    Token::Tab => out_vec.push(AnsiToken::Spaces(4)),
                    Token::Space => out_vec.push(AnsiToken::Spaces(1)),
                    Token::Spaces => {
                        if let Some(new_text) = raw.slice(span.clone()) {
                            out_vec.push(AnsiToken::Spaces(new_text.len()));
                        }
                    }
                    Token::Word => {
                        if let Some(new_text) = raw.slice(span.clone()) {
                            out_vec.push(AnsiToken::Text(String::from(new_text)));
                        }
                    }
                    _ => {}
                }
            },
            // All of these tokens (possibly) change the ANSI state.
            Token::Reset => {
              // Ansi resets work a little bit differently from the rest.
                if ansi_change {
                    // If there has been an ANSI change since the last push that has not yet reached
                    // any test,, then there's no reason to keep it.
                    ansi_change = false;
                    // new_style will not be copied into current_style. Instead, we clear the slate
                    // and push the reset token.
                    out_vec.push(AnsiToken::Reset);
                    current_style = AnsiStyle::default();
                }
            },
            Token::Hilite | Token::Inverse | Token::Blink | Token::Italic |
            Token::Strikethrough | Token::Underline => {
                match tok {
                    Token::Hilite => new_style.hilite = true,
                    Token::Inverse => new_style.inverse = true,
                    Token::Blink => new_style.blink = true,
                    Token::Italic => new_style.italic = true,
                    Token::Strikethrough => new_style.strike = true,
                    Token::Underline => new_style.underline = true,
                    _ => {}
                }
                if !current_style.eq(&new_style) {
                    // If the new style isn't equal to the old, then something changed.
                    // However, many things might change before we start printing more text, so
                    // just note that there has been changes.
                    ansi_change = true;
                    current_style = new_style;
                }
            },
            Token::AnsiFg(val) | Token::XtermFg(val) => {
                new_style.fg = Some(val.clone());
                if !current_style.eq(&new_style) {
                    // If the new style isn't equal to the old, then something changed.
                    // However, many things might change before we start printing more text, so
                    // just note that there has been changes.
                    ansi_change = true;
                    current_style = new_style;
                }
            },
            Token::AnsiBg(val) | Token::XtermBg(val) => {
                new_style.bg = Some(val.clone());
                if !current_style.eq(&new_style) {
                    // If the new style isn't equal to the old, then something changed.
                    // However, many things might change before we start printing more text, so
                    // just note that there has been changes.
                    ansi_change = true;
                    current_style = new_style;
                }
            },
            Token::Error => {}
        }
    }
    out_vec
}


// This is named EvString after Evennia's AnsiString, from whence the ANSI color pattern was taken.
// It is a wrapper around the Thermite AnsiString.
#[derive(Clone, Debug)]
pub struct EvString {
    pub ansi_string: AnsiString,
    pub raw_string: String
}


impl From<&str> for EvString {
    fn from(src: &str) -> Self {
        Self {
            ansi_string: AnsiString::from(parse_evstring(src)),
            raw_string: String::from(src)
        }
    }
}

impl From<String> for EvString {
    fn from(src: String) -> Self {
        Self {
            ansi_string: AnsiString::from(parse_evstring(&src)),
            raw_string: src
        }
    }
}

impl From<EvString> for String {
    fn from(ev: EvString) -> Self {
        ev.raw_string
    }
}

impl EvString {

    pub fn width(&self) -> usize {
        self.ansi_string.width()
    }

}

#[derive(Clone, Debug)]
pub enum Justify {
    Left,
    Right,
    Center
}

#[derive(Clone, Debug)]
pub enum Wrap {
    Truncate,
    WordWrap,
}

#[derive(Clone, Debug)]
pub struct EvCell {
    pub justify: Justify,
    pub wrap: Wrap,
    text: EvString,
    pub width: usize,
}

impl From<EvString> for EvCell {
    fn from(src: EvString) -> Self {
        Self {
            justify: Justify::Left,
            // There is effectively no difference here between 'wrap' and 'no wrap'.
            // Nobody is gonna make a string this big.
            wrap: Wrap::WordWrap,
            width: src.width(),
            text: src
        }
    }
}

impl From<String> for EvCell {
    fn from(src: String) -> Self {
        Self::from(EvString::from(src))
    }
}

impl EvCell {
    pub fn render(&mut self, ansi: bool, xterm256: bool) -> Vec<String> {
        let mut vec_out: Vec<String> = Default::default();

        // We can trust that each line can fit within 'width' characters.
        vec_out
    }

    pub fn contents(&self) -> EvString {
        self.text.clone()
    }
}

#[derive(Clone, Debug)]
pub struct EvFormatRules {
    pub ansi: bool,
    pub xterm256: bool,
    pub width: u16,
    pub height: u16,
}

impl Default for EvFormatRules {
    fn default() -> Self {
        Self {
            ansi: false,
            xterm256: false,
            width: 78,
            height: 24
        }
    }
}

pub trait ToFormatRules {
    fn rules(&self) -> EvFormatRules;
}

impl ToFormatRules for EvFormatRules {
    fn rules(&self) -> EvFormatRules {
        self.clone()
    }
}

#[derive(Clone, Debug)]
pub enum EvFormatRow {
    // When it encounters a FormatRules, EvFormatStack will change its rules.
    FormatRules(EvFormatRules),
    // Text columns are a Vec<EvString>. This is because the number of columns is
    // arbitrary
    Columns(Vec<Option<EvCell>>),
    Header(Option<EvString>),
    Subheader(Option<EvString>),
    Separator(Option<EvString>),
    Text(EvCell),
}

impl EvFormatRow {
    pub fn render(&self, rules: &EvFormatRules) -> String {
        let mut out = String::new();

        match self {
            Self::Text(val) => {
                out.push_str(&val.clone().render(rules.ansi, rules.xterm256).join("\r\n"));
            },
            _ => {

            }
        }

        out
    }
}


#[derive(Clone, Debug)]
pub struct EvFormatStack {
    rows: Vec<EvFormatRow>,
    rules: EvFormatRules,
}

impl EvFormatStack {
    pub fn new(rules: EvFormatRules) -> Self {
        Self {
            rows: Default::default(),
            rules: rules
        }
    }

    pub fn insert(&mut self, row: EvFormatRow) {
        self.rows.push(row);
    }

    pub fn clear(&mut self) {
        self.rows.clear();
    }

    pub fn render(&self) -> String {
        let mut cur_rules = self.rules.clone();
        let mut out = String::new();

        for row in self.rows.iter() {
            if let EvFormatRow::FormatRules(new) = row {
                cur_rules = new.clone()
            }
            else {
                out.push_str(&row.render(&cur_rules))
            }
        }
        out
    }
}