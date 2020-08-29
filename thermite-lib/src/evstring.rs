use crate::ansi::{
    ColorCode,
    AnsiSpan
};

use logos::{Logos, Lexer, Span, Source};


use std::{
    str::FromStr,
    collections::HashMap
};

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


// This is named EvString after Evennia, from whence the ANSI color pattern was taken.
pub struct EvString {
    pub raw: String,
    pub plain: String
}

impl FromStr for EvString {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_string = String::from(s);
        let out = Self::from(raw_string);
        Ok(out)
    }
}

impl From<String> for EvString {
    fn from(raw: String) -> Self {
        Self {
            plain: Self::make_plain(&raw),
            raw
        }
    }
}

impl EvString {
    pub fn new(src: String) -> Self {
        Self::from(src)
    }

    fn make_plain(src: &str) -> String {
        let mut plain = String::new();

        for (tok, span) in Token::lexer(src).spanned() {
            match tok {
                Token::Pipe => plain.push_str("|"),
                Token::Crlf => plain.push_str("\r\n"),
                Token::Tab => plain.push_str("\t"),
                Token::Space => plain.push_str(" "),
                Token::Spaces | Token::Word => {
                    if let Some(new_text) = &src.slice(span.clone()) {
                        plain.push_str(new_text);
                    }
                }
                _ => {}
            }
        }
        plain
    }

    pub fn len(&self) -> usize {
        self.raw.len() + self.plain.len()
    }

    pub fn width(&self) -> usize {
        self.plain.chars().count()
    }

    pub fn render(&self, ansi: bool, xterm256: bool) -> String {
        // if xterm256 is enabled, this will force-enable ansi.
        let mut use_ansi = true;
        if xterm256 {
            use_ansi = true
        }

        if !use_ansi {
            // There is no reason to render ANYTHING if there's no ANSI, so just
            // return plain.
            return self.plain.clone();
        }

        let mut parent_aspan = AnsiSpan::new();
        let mut out = String::new();

        for (tok, span) in Token::lexer(&self.raw).spanned() {
            let mut child_aspan = parent_aspan.clone();

            match tok {
                Token::Pipe => out.push_str("|"),
                Token::Crlf => out.push_str("\r\n"),
                Token::Tab => out.push_str("\t"),
                Token::Space => out.push_str(" "),
                Token::Spaces | Token::Word => {
                    if let Some(new_text) = self.raw.slice(span.clone()) {
                        out.push_str(new_text);
                    }
                },
                Token::Reset => child_aspan.reset(),
                Token::Hilite => child_aspan.hilite = true,
                Token::Inverse => child_aspan.inverse = true,
                Token::Blink => child_aspan.blink = true,
                Token::Italic => child_aspan.italic = true,
                Token::Strikethrough => child_aspan.strike = true,
                Token::Underline => child_aspan.underline = true,
                Token::AnsiFg(val) => child_aspan.fg = Some(val.clone()),
                Token::AnsiBg(val) => child_aspan.bg = Some(val.clone()),
                Token::XtermBg(val) => {
                    if !xterm256 {
                        // Not sure what to do here yet...
                    } else {
                        child_aspan.bg = Some(val.clone())
                    }
                },
                Token::XtermFg(val) => {
                    if !xterm256 {
                        // Not sure what to do here yet...
                    } else {
                        child_aspan.fg = Some(val.clone())
                    }
                }
                _ => {}
            }

            if !parent_aspan.eq(&child_aspan) {
                out.push_str(&child_aspan.difference(&child_aspan));
                parent_aspan = child_aspan;
            }

        }
        out
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
    ColumnNames(Vec<EvString>),
    Columns(Vec<EvString>),
    Header(Option<EvString>),
    Subheader(Option<EvString>),
    Separator(Option<EvString>),
    Text(Option<EvString>),
}

impl EvFormatRow {
    pub fn render(&self, rules: &EvFormatRules) -> String {
        let mut out = String::new();

        match self {
            Self::Text(val) => {
                if let Some(val) = val {
                    out.push_str(&val.render(rules.ansi, rules.xterm256));
                }
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
    pub fn new(rules: &impl ToFormatRules) -> Self {
        Self {
            rows: Default::default(),
            rules: rules.rules()
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