

use std::{
    str::FromStr,
    collections::HashMap
};

use regex::Regex;

pub fn strip_ansi(src: &str) -> String {
    let stripper = Regex::new("\x1b\\[\\S*?m").unwrap();
    String::from(stripper.replace_all(src, ""))
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
