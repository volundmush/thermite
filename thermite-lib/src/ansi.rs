
use std::{
    str::FromStr,
    collections::HashMap,
    iter::repeat
};

use regex::Regex;
use futures::StreamExt;
use logos::{Logos, Lexer, Span, Source};
//use unicode_segmentation::UnicodeSegmentation;

use crate::repeat_string;

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


#[derive(Clone, Debug)]
pub enum Wrapping {
    Word,
    Letter,
}

#[derive(Clone, Debug)]
pub struct AnsiRenderRules {
    pub ansi: bool,
    pub xterm256: bool,
    pub line_length: usize,
    pub max_lines: usize,
    pub wrap_style: Wrapping,
    pub ansi_style: Option<AnsiStyle>
}

impl Default for AnsiRenderRules {
    fn default() -> Self {
        Self {
            ansi: false,
            xterm256: false,
            line_length: usize::MAX,
            max_lines: usize::MAX,
            wrap_style: Wrapping::Word,
            ansi_style: None
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct AnsiLine {
    pub text: String,
    pub length: usize
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

    pub fn render_lines(&self, rules: &AnsiRenderRules) -> Vec<AnsiLine> {
        let mut out_vec: Vec<AnsiLine> = Default::default();
        let mut current_style = rules.ansi_style.clone().unwrap_or_default();
        let use_ansi = (rules.ansi || rules.xterm256);
        let mut remaining_length: usize = 0;
        let mut ansi_changed = false;

        let mut token_line: Vec<AnsiToken> = Default::default();
        let mut token_lines: Vec<Vec<AnsiToken>> = Default::default();

        // For this first iteration, we are going to handle word wrapping/truncation and max
        // lines.
        for tok in self.tokens.iter() {
            let tok_len = tok.len();
            match tok {
                AnsiToken::Text(val) => {
                    // This is guaranteed to be a bunch of non-ansi, non-whitespace. It's
                    // probably a whole world, or hyphenated-underscored-weirdness.
                    // If there's enough room to fit this on the current line, then pass the
                    // token on directly. If not, we need to split it...
                    if tok_len <= remaining_length {
                        token_line.push(tok.clone());
                        remaining_length = remaining_length - tok_len;
                    } else {
                        // In this case, we're going to have to split it and pass new tokens.
                        // There are two different logical approaches.
                        let mut break_again = false;
                        match rules.wrap_style {
                            Wrapping::Letter => {
                                // In letter style, a character is a character, just as if it were
                                // a space. This can mean that words are split up casually.
                                let mut piece = val.slice(0..val.len()).unwrap();
                                if remaining_length > 0 {
                                    // If there is any remaining length we need to append whatever we have...
                                    // Am just unwrapping. It's impossible for this to panic.
                                    piece = val.slice(0..remaining_length).unwrap();
                                    token_line.push(AnsiToken::Text(String::from(piece)));
                                    remaining_length = rules.line_length;
                                    token_lines.push(token_line.clone());
                                    token_line.clear();
                                    if token_lines.len() >= rules.max_lines {
                                        break;
                                    }
                                }
                                for chr in piece.chars() {


                                    if remaining_length == 0 {

                                    }
                                }
                                for i in 0..piece.len()/rules.line_length {
                                    if let Some(sli) = piece.slice() {
                                        token_line.push(AnsiToken::Text(String::from(sli)));
                                        token_lines.push(token_line.clone());
                                        token_line.clear();
                                        remaining_length = rules.line_length - sli.len();
                                        if token_lines.len() >= rules.max_lines {
                                            // Since we are already INSIDE a for loop, then we need to break the
                                            // outer one.
                                            break_again = true;
                                            break;
                                        }
                                    }
                                }
                                if break_again {
                                    break;
                                }
                            },
                            Wrapping::Word => {
                                // In Word style, a word that cannot fit on remaining space will
                                // instead begin on the next line. If it still can't fit, then we
                                // progress in the same manner as Letter style.
                                // If this pushes us immediately beyond max lines, oh well... this
                                // word won't be shown.
                                token_lines.push(token_line.clone());
                                token_line.clear();
                                if token_lines.len() >= rules.max_lines {
                                    break;
                                }
                                for p in val.as_bytes().chunks(rules.line_length) {
                                    token_line.push(AnsiToken::Text(String::from(p)));
                                    token_lines.push(token_line.clone());
                                    token_line.clear();
                                    remaining_length = rules.line_length - p.len();
                                    if token_lines.len() >= rules.max_lines {
                                        // Since we are already INSIDE a for loop, then we need to break the
                                        // outer one.
                                        break_again = true;
                                        break;
                                    }
                                }
                                if break_again {
                                    break;
                                }
                            }
                        }
                    }
                },
                AnsiToken::Spaces(len) => {
                    // This is basically the exact same process as with text.
                    if tok_len <= remaining_length {
                        token_line.push(tok.clone());
                        remaining_length = remaining_length - tok_len;
                    } else {
                        let mut spaces_left = tok_len;
                        // In this case, we're going to have to split it and pass new tokens.
                        if remaining_length > 0 {
                            // If there is any remaining length we need to append whatever we have...
                            // Am just unwrapping. It's impossible for this to panic.
                            token_line.push(AnsiToken::Spaces(remaining_length));
                            spaces_left = spaces_left - remaining_length;
                            remaining_length = rules.line_length;
                            token_lines.push(token_line.clone());
                            token_line.clear();
                            if token_lines.len() >= rules.max_lines {
                                break;
                            }
                        }
                        let mut break_again = false;
                        // The process for spaces is to simply insert (spaces_left / line_length)
                        // lines, then a token with the modulo. Easy.

                        for _ in 1..spaces_left/rules.line_length {
                            token_line.push(AnsiToken::Spaces(rules.line_length));
                            token_lines.push(token_line.clone());
                            token_line.clear();
                            if token_lines.len() >= rules.max_lines {
                                // Since we are already INSIDE a for loop, then we need to break the
                                // outer one.
                                break_again = true;
                                break;
                            }
                        }
                        if break_again {
                            break;
                        }
                        let remainder = spaces_left%rules.line_length;

                        if remainder > 0 {
                            token_line.push(AnsiToken::Spaces(remainder));
                            token_lines.push(token_line.clone());
                            token_line.clear();
                        }
                    }
                },
                AnsiToken::Newline => {
                    token_lines.push(token_line.clone());
                    token_line.clear();
                    remaining_length = rules.line_length;
                    if token_lines.len() >= rules.max_lines {
                        break;
                    }
                },
                AnsiToken::Ansi(style) => {
                    // Every AnsiStyle that's received will lazily wait until text or spaces
                    // before it is printed. This is done to prevent wasting ANSI prints at
                    // the end of lines.
                    if use_ansi {
                        token_line.push(tok.clone());
                    }
                },
                AnsiToken::Reset => {
                    if use_ansi {
                        token_line.push(tok.clone());
                    }
                }
            }
        }
        if token_line.len() > 0 {
            // Seems there was leftover input in this one.
            token_lines.push(token_line.clone());
            token_line.clear()
        }

        // Now that we have formatted all of the text properly, we iterate once more to fill out_vec
        // with ansilines. At this point, we no longer need to care about whether the line is the
        // right size. Just generate the ANSI text and 'apparent' length and stuff it into an
        // AnsiLine.
        for line in token_lines.iter() {
            let mut out_str = String::new();
            let mut line_length: usize = 0;


            if use_ansi {
                // Each new line should inherit the current ANSI style if using ansi.
                out_str.push_str(&current_style.render(rules.xterm256, true));
            }

            for tok in line.iter() {
                // We will be ignoring newlines since that's what each vector IS...
                line_length = line_length + tok.len();
                match tok {
                    AnsiToken::Ansi(style) => {
                        // If use_ansi is disabled, the vector won't even contain Ansi or Reset.
                        if !current_style.eq(style) {
                            out_str.push_str(&current_style.difference(style, rules.xterm256));
                            current_style = style.clone();
                        }
                    },
                    AnsiToken::Text(val) => {
                        out_str.push_str(val);
                    },
                    AnsiToken::Spaces(len) => {
                        out_str.push_str(&repeat_string(" ", len.clone()));
                    },
                    AnsiToken::Reset => {
                        current_style = AnsiStyle::default();
                        out_str.push_str("\x1b[0m");
                    },
                    _ => {}
                }
            }

            if use_ansi {
                // When using ANSI, terminate each line with an ansi reset.
                out_str.push_str("\x1b[0m");
            }

            out_vec.push(AnsiLine {
                text: out_str,
                length: line_length
            })
        }
        out_vec
    }
}