use logos::{Logos, Lexer, Span, Source};

use std::{
    str::FromStr,
    collections::HashMap
};

pub const ESC: &str = "\x1b";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorCode {
    Ansi(u8),
    Xterm(u8)
}

impl ColorCode {
    pub fn as_fg(&self) -> String {
        match self {
            Self::Ansi(val) => format!("{}", val),
            Self::Xterm(val) => format!("38;5;{}", val)
        }
    }

    pub fn as_bg(&self) -> String {
        match self {
            Self::Ansi(val) => format!("{}", val),
            Self::Xterm(val) => format!("48;5;{}", val)
        }
    }

    pub fn as_ansi(&self) -> Self {
        match self {
            Self::Ansi(val) => self.clone(),
            Self::Xterm(val) => {
                // This should degrade an Xterm color to the nearest ansi color.
                // Right now it doesn't.
                Self::Ansi(30)
            }
        }
    }
}

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

#[derive(Debug)]
pub enum AnsiPiece {
    Word(String),
    Beep,
    Space,
    Crlf,
    Tab
}

// This struct represents a segment of text wrapped by ANSI encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnsiSpan {
    pub fg: Option<ColorCode>,
    pub bg: Option<ColorCode>,
    pub hilite: bool,
    pub dim: bool,
    pub italic: bool,
    pub inverse: bool,
    pub underline: bool,
    pub blink: bool,
    pub conceal: bool,
    pub strike: bool,
}

impl Default for AnsiSpan {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            hilite: false,
            dim: false,
            underline: false,
            blink: false,
            strike: false,
            inverse: false,
            italic: false,
            conceal: false
        }
    }
}

impl AnsiSpan {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.fg = None;
        self.bg = None;
        self.hilite = false;
        self.dim = false;
        self.underline = false;
        self.blink = false;
        self.strike = false;
        self.inverse = false;
        self.italic = false;
        self.conceal = false;
    }

    pub fn render(&self) -> String {
        String::new()
    }

    pub fn codes(&self) -> String {
        // Generates just the ANSI color codes for this AnsiSpan, minus the rest
        // of ANSI formatting
        let mut segments: Vec<String> = Vec::with_capacity(10);

        if self.inverse {
            segments.push(String::from("7"))
        }

        if self.hilite {
            segments.push(String::from("1"))
        }

        if self.dim {
            segments.push(String::from("2"))
        }

        if self.underline {
            segments.push(String::from("4"))
        }

        if self.strike {
            segments.push(String::from("9"))
        }

        if self.italic {
            segments.push(String::from("3"))
        }

        if self.blink {
            segments.push(String::from("5"))
        }

        if self.conceal {
            segments.push(String::from("8"))
        }

        if let Some(code) = &self.fg {
            segments.push(code.as_fg())
        }

        if let Some(code) = &self.bg {
            segments.push(code.as_bg())
        }

        let mut out = segments.join(";");
        out.push_str("m");
        out
    }

    pub fn difference(&self, other: &Self) -> String {
        // In Left to Right order, compares self with other and generates a
        // ANSI sequence that applies changes. This will minimize text generation
        // IE: this attempts to create a 'delta' of ANSI codes in-between states of
        // text generation.

        // First, hilite, strike, inverse, italic, and underline cannot be 'canceled',
        // only reset. If other 'disables' them, we can only reset.
        // In that case, the answer is to return the codes() of other.
        if (!other.hilite && self.hilite)
            || (!other.strike && self.strike)
            || (!other.inverse && self.inverse)
            || (!other.underline && self.underline)
            || (!other.italic && self.italic)
            || (!other.dim && self.dim)
            || (!other.conceal && self.conceal ) {
            return format!("\x1b[0;{}", other.codes());
        } else {
            // But if all we need to do is change color codes or add new font
            // options, we can do that in-line.
            let mut segments: Vec<String> = Vec::with_capacity(2);

            if !self.inverse && other.inverse {
                segments.push(String::from("7"))
            }

            if !self.hilite && other.hilite {
                segments.push(String::from("1"))
            }

            if !self.dim && other.dim {
                segments.push(String::from("2"))
            }

            if !self.underline && other.underline {
                segments.push(String::from("4"))
            }

            if !self.strike && other.strike {
                segments.push(String::from("9"))
            }

            if !self.italic && other.italic {
                segments.push(String::from("3"))
            }

            if !self.blink && other.blink {
                segments.push(String::from("5"))
            }

            if !self.conceal && other.conceal {
                segments.push(String::from("8"))
            }

            if let Some(code) = &self.fg {
                segments.push(code.as_fg())
            }

            if let Some(code) = &self.bg {
                segments.push(code.as_bg())
            }
            format!("\x1b[{}m", segments.join(";"))
        }
    }
}

pub struct ANSIString {
    pub raw: String,
    pub plain: String
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
    fn from(raw: String) -> Self {
        Self {
            plain: Self::make_plain(&raw),
            raw
        }
    }
}

impl ANSIString {
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