use crate::{
    softcode::typedefs::{DbRef, DbError}
};

use std::{
    io::{Read, BufRead, Lines, Bytes},
    error::Error,
    iter::Iterator,
    fs::File,
    rc::Rc
};

use encoding_rs::*;
use encoding_rs_io::*;
use std::path::Path;
use std::fmt::Display;
use serde::export::Formatter;

#[derive(Debug, Clone)]
pub enum NodeValue {
    None,
    Text(String),
    Db(DbRef),
    Number(isize),
}

impl From<NodeValue> for String {
    fn from(src: NodeValue) -> Self {
        match src {
            NodeValue::Text(txt) => txt,
            NodeValue::Db(dbr) => dbr.to_string(),
            NodeValue::Number(num) => format!("{}", num),
            NodeValue::None => "".to_string()
        }
    }
}

impl NodeValue {
    pub fn as_str(&self) -> &str {
        match self {
            NodeValue::Text(txt) => txt.as_str(),
            _ => ""
        }
    }

    pub fn is_str(&self) -> bool {
        match self {
            NodeValue::Text(txt) => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlatValueNode {
    pub name: String,
    pub depth: usize,
    pub value: NodeValue,
}

impl Default for FlatValueNode {
    fn default() -> Self {
        Self {
            name: Default::default(),
            depth: 0,
            value: NodeValue::None
        }
    }
}

impl FlatValueNode {
    pub fn as_str(&self) -> &str {
        self.value_str()
    }

    pub fn value_str(&self) -> &str {
        self.value.as_str()
    }
}

impl From<String> for FlatValueNode {
    fn from(src: String) -> Self {
        Self::from(src.as_str())
    }
}

impl From<&str> for FlatValueNode {
    fn from(src: &str) -> Self {

        let depth = count_spaces(src);
        let (spaces, data) = src.split_at(depth);
        let mut split= data.splitn(2, " ");
        let name = split.next().unwrap().to_string();
        let val = split.next().unwrap();

        if val.starts_with('"') && val.ends_with('"') {
            // This is a string value. We want everything except the beginning and ending quotes.
            let out = &val[1..val.len()-1];
            return Self {
                name,
                value: NodeValue::Text(out.to_string()),
                depth,
            }
        }

        if val.starts_with("#") {
            let db: DbRef = if val.starts_with("#-") {
                // this is a null Dbref.
                DbRef::None
            } else {
                let (_, num) = val.split_at(1);
                DbRef::Num(num.parse::<usize>().unwrap())
            };
            return Self {
                name,
                value: NodeValue::Db(db),
                depth,
            }
        }

        // The only option left is numeric.
        let num = val.parse::<isize>().unwrap();
        return Self {
            name,
            value: NodeValue::Number(num),
            depth,
        }
    }
}

#[derive(Debug)]
pub enum FlatLine {
    Header(String),
    Node(FlatValueNode)
}

impl From<String> for FlatLine {
    fn from(src: String) -> Self {
        if src.starts_with('+')
            || src.starts_with('!')
            || src.starts_with('~')
            || src.starts_with('*') {
            Self::Header(src.to_string())
        } else {
            Self::Node(FlatValueNode::from(src))
        }
    }
}

impl FlatLine {
    pub fn is_header(&self) -> bool {
        match self {
            Self::Header(txt) => {
                true
            },
            _ => {
                false
            }
        }
    }

    pub fn is_node(&self) -> bool {
        return !self.is_header()
    }

    pub fn is_str(&self) -> bool {
        match self {
            Self::Node(val) => {
                val.value.is_str()
            },
            _ => {
                false
            }
        }
    }

    pub fn name_str(&self) -> &str {
        match self {
            Self::Header(txt) => {
                txt.as_str()
            },
            Self::Node(val) => {
                val.name.as_str()
            }
        }
    }

    pub fn value_str(&self) -> &str {
        match self {
            Self::Header(txt) => {
                txt.as_str()
            },
            Self::Node(val) => {
                val.value_str()
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Header(txt) => {
                txt.as_str()
            },
            Self::Node(val) => {
                val.as_str()
            }
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            Self::Header(txt) => {
                0
            },
            Self::Node(val) => {
                val.depth
            }
        }
    }

    pub fn dbref(&self, name: &str, err: &str) -> Result<DbRef, DbError> {
        match self {
            Self::Node(val) => {
                if val.name.starts_with(name) {
                    match &val.value {
                        NodeValue::Db(t) => {
                            Ok(t.clone())
                        },
                        NodeValue::Text(s) => Ok(DbRef::Name(s.to_uppercase())),
                        _ => {
                            return Err(DbError::new(format!("{}: {}", err, name).as_str()).into());
                        }
                    }
                } else {
                    return Err(DbError::new(format!("{}: {}", err, name).as_str()).into());
                }
            },
            _ => {
                return Err(DbError::new(format!("{}: {}", err, name).as_str()).into());
            }
        }
    }

    pub fn text(&self, name: &str, err: &str) -> Result<String, DbError> {

        match self {
            Self::Node(val) => {
                if val.name.starts_with(name) {
                    match &val.value {
                        NodeValue::Text(t) => {
                            Ok(t.clone())
                        },
                        _ => {
                            return Err(DbError::new(format!("{}: {}", err, name).as_str()).into());
                        }
                    }
                } else {
                    return Err(DbError::new(format!("{}: {}", err, name).as_str()).into());
                }
            },
            _ => {
                return Err(DbError::new(format!("{}: {}", err, name).as_str()).into());
            }
        }

    }

    pub fn num(&self, name: &str, err: &str) -> Result<isize, DbError> {

        match self {
            Self::Node(val) => {
                if val.name.starts_with(name) {
                    match &val.value {
                        NodeValue::Number(t) => {
                            Ok(t.clone())
                        },
                        _ => {
                            return Err(DbError::new(format!("{}: {}", err, name).as_str()).into());
                        }
                    }
                } else {
                    return Err(DbError::new(format!("{}: {}", err, name).as_str()).into());
                }
            },
            _ => {
                return Err(DbError::new(format!("{}: {}", err, name).as_str()).into());
            }
        }

    }
}

pub fn count_spaces(line: &str) -> usize {
    let mut space_count: usize = 0;
    for c in line.chars() {
        if c == ' ' {
            space_count += 1;
        } else {
            break;
        }
    }
    space_count
}


// This is designed to read PennMUSH's strange flatfile format and turn any lines which
// include 'quoted strings' into single strings.
pub struct FlatFileSplitter<T> {
    source: Bytes<T>,
    quoted: bool,
    escaped: bool
}

impl<T> FlatFileSplitter<T> where T: Read {
    pub fn new(source: T) -> Self {
        Self {
            source: source.bytes(),
            quoted: false,
            escaped: false
        }
    }
}


impl<T> Iterator for FlatFileSplitter<T> where T: Read {
    type Item = std::io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        // Consume bytes from source until we hit a newline. If we ever encounter a double-quote,
        // enter quoted mode and keep consuming until we encounter an unescaped closing double-quote,
        // then keep watching for newlines.
        let mut buffer: Vec<u8> = Default::default();

        loop {
            if let Some(res) = self.source.next() {
                match res {
                    Ok(c) => {
                        if self.quoted {
                            if self.escaped {
                                buffer.push(c);
                                self.escaped = false;
                            } else {
                                match c {
                                    92 => {
                                        self.escaped = true;
                                    },
                                    34 => {
                                        self.quoted = false;
                                        buffer.push(c);
                                    },
                                    _ => {
                                        buffer.push(c);
                                    }
                                }
                            }
                        } else {
                            match c {
                                13 => {
                                    // We just ignore this character outside of quoted strings.
                                },
                                10 => {
                                    // Newline detected outside quotes! Convert buffer to a &str and shove it out.
                                    match String::from_utf8(buffer) {
                                        Ok(out) => {
                                            return Some(Ok(out));
                                        },
                                        Err(e) => {
                                            // If something goes wrong... return an error instead. All data for this line is
                                            // lost. Of course, we probably stop parsing here anyways.
                                            return Some(Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)));
                                        }
                                    }
                                },
                                34 => {
                                    // Begin quoted string.
                                    self.quoted = true;
                                    buffer.push(c);
                                },
                                _ => {
                                    buffer.push(c);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        // this is some kind of io error that isn't an EOF.
                        return Some(Err(e));
                    }
                }
            } else {
                // We hit an EOF.
                return None;
            }
        }

    }
}



pub struct FlatFileReader<T> {
    source: T
}

impl<T> FlatFileReader<T> where
    T: Iterator<Item=std::io::Result<String>> {
    pub fn new(source: T) -> Self {
        Self {
            source
        }
    }
}

impl<T> Iterator for FlatFileReader<T> where
    T: Iterator<Item=std::io::Result<String>>
{
    type Item = std::io::Result<FlatLine>;

    fn next(&mut self) -> Option<Self::Item> {
        // Take the output of our provided FlatFileSplitter and
        // convert each &str into a FlatValueNode.

        if let Some(res) = self.source.next() {
            match res {
                Ok(txt) => {
                    return Some(Ok(FlatLine::from(txt)));
                },
                Err(e) => {
                    return Some(Err(e));
                }
            }
        } else {
            // We are at an end.
            return None;
        }
    }
}