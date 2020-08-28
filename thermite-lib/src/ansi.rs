use logos::{Logos, Lexer, Span};

use std::{
    str::FromStr,
    collections::HashMap
};

fn getstr(lex: &mut Lexer<Token>) -> Option<String> {
    Some(String::from(lex.slice()))
}


#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[regex("\\|n", getstr)]
    AnsiReset(String),

    #[regex("\\|[urgybmcwxRGYBMCWXhH]", getstr)]
    AnsiFg(String),

    #[regex("\\|[\\\\/\\-_\\*\\^]", getstr)]
    AnsiCode(String),

    #[regex("\\|![RGYBMCWX]", getstr)]
    AnsiHilite(String),

    #[regex("\\|\\[[RGYBMCWX]", getstr)]
    AnsiBg(String),

    #[regex("\\|\\[[rgybmcwx]", getstr)]
    XtermHiBg(String),

    #[regex("\\|([0-5])([0-5])([0-5])", getstr)]
    XtermFgNum(String),

    #[regex("\\|\\[([0-5])([0-5])([0-5])", getstr)]
    XtermBgNum(String),

    #[regex("\\|=([a-z])", getstr)]
    XtermGfgAz(String),

    #[regex("\\|\\[=([a-z])", getstr)]
    XtermGbgAz(String),

    #[token(" ")]
    Space,

    #[token("||")]
    Pipe,

    #[regex("[^\\s\\|]+", getstr)]
    Word(String),

    #[error]
    Error
}

pub enum AnsiToken {
    AnsiCode(String),
    AnsiReset,
    Word(String),
    CRLF,
    Tab,
    Space,
    Error
}

impl FromStr for AnsiToken {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_string = String::from(s);
        let out = Self::from(raw_string);
        Ok(out)
    }
}

impl From<String> for AnsiToken {
    fn from(src: String) -> Self {

        if src.len() < 2 {
            return Self::Error;
        }

        if &src == "|n" {
            return Self::AnsiReset;
        }

        if &src == "|\\" {
            return Self::CRLF;
        }

        if &src == "|-" {
            return Self::Tab;
        }

        if &src == "|*" {
            //return Self::Blink;
        }

        if &src == "|u" {
            // return Self::Underline;
        }

        if src.starts_with("|!") {

        }

        if src.starts_with("|[") {

        }

        Self::Word(src)
    }
}

pub struct ANSIString {
    clean_string: String,
    raw_string: String,
    ansi_string: String,
    tokens: Vec<(Token, Span)>,
    ansi_tokens: Vec<AnsiToken>
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

        let tokens: Vec<(Token, Span)> = lexer.spanned().collect();
        let mut atokens: Vec<AnsiToken> = Vec::with_capacity(tokens.len());

        let mut clean = String::new();
        let mut ansi = String::new();
        for (tok, span) in tokens.iter() {
            match tok {
                Token::Space => {
                    clean.push_str(" ");
                    ansi.push_str(" ");
                },
                Token::Word(val) => {
                    clean.push_str(&val);
                    ansi.push_str(&val);
                },
                Token::Pipe => {
                    clean.push_str("|");
                    ansi.push_str("|");
                }
                _ => {

                }
            }
        }


        Self {
            ansi_string: None,
            clean_string: None,
            raw_string: src,
            tokens
        }
    }
}

impl ANSIString {
    pub fn new(src: String) -> Self {
        Self::from(src)
    }

    pub fn plain(&self) -> String {
        return self.clean_string.clone();
    }
}