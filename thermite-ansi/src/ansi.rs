
use std::{
    collections::HashMap,
    iter::repeat
};

use regex::Regex;

use vtparse::{
    VTActor, VTParser
};

use hyphenation::{Language, Load, Standard};
use textwrap::Wrapper;

use thermite_util::text::{repeat_string};

//use unicode_segmentation::UnicodeSegmentation;

pub fn strip_ansi(src: &str) -> String {
    let stripper = Regex::new("\x1b\\[\\S*?m").unwrap();
    String::from(stripper.replace_all(src, ""))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnsiToken {
    Ansi(AnsiStyle),
    Text(String),
    Spaces(usize),
    Newline,
    Reset
}

impl AnsiToken {
    pub fn len(&self) -> usize {
        match self {
            Self::Text(val) => val.chars().count(),
            Self::Spaces(val) => val.clone(),
            Self::Newline => 1,
            _ => 0
        }
    }

    pub fn print(&self, rules: &AnsiRenderRules) -> String {
        match self {
            Self::Reset => String::from("\x1b[0m"),
            Self::Spaces(val) => repeat_string(" ", val),
            Self::Newline => String::from("\n"),
            Self::Text(val) => val.clone(),
            Self::Ansi(style) => {
                if !rules.ansi {
                    String::new()
                } else {
                    style.render(rules.xterm256, false)
                }
            }.
        }
    }
}

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

// This struct represents a segment of text wrapped by ANSI encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnsiStyle {
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

impl Default for AnsiStyle {
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

impl AnsiStyle {

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

    pub fn render(&self, xterm256: bool, reset: bool) -> String {
        if reset {
            format!("\x1b[0;{}", self.codes(xterm256))
        } else {
            format!("\x1b[{}", self.codes(xterm256))
        }
    }

    pub fn codes(&self, xterm256: bool) -> String {
        // Generates just the ANSI color codes for this AnsiStyle, minus the rest
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

    pub fn difference(&self, other: &Self, xterm256: bool) -> String {
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
            || (!other.conceal && self.conceal )
            || (self.fg != None && other.fg == None)
            || (self.bg != None && other.fg == None) {
            return format!("\x1b[0;{}", other.codes(xterm256));
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

            if let Some(code) = &other.fg {
                segments.push(code.as_fg())
            }

            if let Some(code) = &other.bg {
                segments.push(code.as_bg())
            }
            format!("\x1b[{}m", segments.join(";"))
        }
    }
}

// We only care about CsiDispatch for AnsiStyle. Everythign else will be ignored.
impl VTActor for AnsiStyle {
    fn print(&mut self, b: char) {}

    fn execute_c0_or_c1(&mut self, control: u8) {}

    fn dcs_hook(&mut self, params: &[i64], intermediates: &[u8], ignored_excess_intermediates: bool) {}

    fn dcs_put(&mut self, byte: u8) {}

    fn dcs_unhook(&mut self) {}

    fn esc_dispatch(&mut self, params: &[i64], intermediates: &[u8], ignored_excess_intermediates: bool, byte: u8) {}

    fn osc_dispatch(&mut self, params: &[&[u8]]) {}

    fn csi_dispatch(&mut self, params: &[i64], intermediates: &[u8], ignored_excess_intermediates: bool, byte: u8) {
        let mut state = AnsiParseState::Normal;

        for i in params.iter() {
            let num = *i as u8;
            match state {
                AnsiParseState::Normal => {
                    match num {
                        0 => self.reset(),
                        1 => self.hilite = true,
                        2 => self.dim = true,
                        3 => self.italic = true,
                        4 => self.underline = true,
                        5 => self.blink = true,
                        8 => self.conceal = true,
                        9 => self.strike = true,
                        30..37 => self.fg = Some(ColorCode::Ansi(num.clone())),
                        40..47 => self.bg = Some(ColorCode::Ansi(num.clone())),
                        38 => state = AnsiParseState::XtermFg1,
                        48 => state = AnsiParseState::XtermBg1,
                        _ => {}
                    }

                },
                AnsiParseState::XtermBg1 => {
                    match num {
                        5 => state = AnsiParseState::XtermBg2,
                        _ => state = AnsiParseState::Normal
                    }
                },
                AnsiParseState::XtermBg2 => {
                    self.bg = Some(ColorCode::Xterm(num));
                    state = AnsiParseState::Normal;
                },
                AnsiParseState::XtermFg1 => {
                    match num {
                        5 => state = AnsiParseState::XtermFg2,
                        _ => state = AnsiParseState::Normal
                    }
                },
                AnsiParseState::XtermFg2 => {
                    self.fg = Some(ColorCode::Xterm(num));
                    state = AnsiParseState::Normal;
                }
            }
        }
    }
}

enum AnsiParseState {
    Normal,
    XtermBg1,
    XtermBg2,
    XtermFg1,
    XtermFg2
}

#[derive(Clone, Debug)]
pub enum Wrapping {
    Word,
    Letter,
}

#[derive(Clone, Debug)]
pub struct AnsiRenderRules {
    pub ansi: bool,
    pub xterm256: bool,
    pub max_width: usize,
    pub max_lines: usize,
    pub wrap_style: Wrapping,
    pub ansi_style: Option<AnsiStyle>
}

impl Default for AnsiRenderRules {
    fn default() -> Self {
        Self {
            ansi: false,
            xterm256: false,
            max_width: usize::MAX,
            max_lines: usize::MAX,
            wrap_style: Wrapping::Word,
            ansi_style: None
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct AnsiLine {
    pub text: String,
    pub width: usize
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnsiString {
    pub tokens: Vec<AnsiToken>
}

impl From<Vec<AnsiToken>> for AnsiString {
    fn from(tokens: Vec<AnsiToken>) -> Self {
        Self {
            tokens
        }
    }
}


impl AnsiString {
    pub fn width(&self) -> usize {
        self.tokens.iter().map(|t| t.len()).sum()
    }

    fn render(&self, rules: &AnsiRenderRules) -> AnsiLine {
        // Simply converts Tokens into ANSI
        let mut out = String::new();

        for tok in self.tokens.iter() {

        }

    }

    pub fn render_lines(&self, rules: &AnsiRenderRules) -> Vec<AnsiLine> {
        let mut out_vec: Vec<AnsiLine> = Default::default();
        let mut current_style = rules.ansi_style.clone().unwrap_or_default();
        let use_ansi = (rules.ansi || rules.xterm256);



        let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
        let wrapper = Wrapper::with_splitter(rules.line_length, dictionary);

        out_vec
    }
}