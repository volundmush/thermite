use super::{
    core::DbError,
    typedefs::Dbref
};

use std::{
    io::{Read, BufRead, Lines, Bytes},
    error::Error,
    iter::Iterator
};

#[derive(Debug, Clone)]
pub enum NodeValue {
    None,
    Text(String),
    Db(Dbref),
    Number(isize),
}

impl From<NodeValue> for String {
    fn from(src: NodeValue) -> Self {
        match src {
            NodeValue::Text(txt) => txt,
            NodeValue::Db(dbr) => format!("#{}", dbr),
            NodeValue::Number(num) => format!("{}", num),
            NodeValue::None => "".to_string()
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
            // this is a dbref value. parse it!
            let (unused, num) = val.split_at(1);
            let count = num.parse::<Dbref>().unwrap();
            return Self {
                name,
                value: NodeValue::Db(count),
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

pub enum FlatLine {
    Header(String),
    Node(FlatValueNode)
}

impl From<&str> for FlatLine {
    fn from(src: &str) -> Self {
        if src.starts_with('+') {
            Self::Header(src.to_string())
        } else {
            Self::Node(FlatValueNode::from(src))
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


impl Iterator for FlatFileSplitter<T> {
    type Item = std::io::Result<&str>;

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
                                self.buffer.push(c);
                                self.escaped = false;
                            } else {
                                match c {
                                    '\\' => {
                                        self.escaped = true;
                                    },
                                    '"' => {
                                        self.quoted = false;
                                        self.buffer.push(c);
                                    },
                                    _ => self.buffer.push(c);
                                }
                            }
                        } else {
                            match c {
                                '\r' => {
                                    // We just ignore this character outside of quoted strings.
                                },
                                '\n' => {
                                    // Newline detected outside quotes! Convert buffer to a &str and shove it out.
                                    match str::from_utf8(self.buffer.as_slice()) {
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
                                '"' => {
                                    // Begin quoted string.
                                    self.quoted = true;
                                    self.buffer.push(c);
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
    T: Iterator<Item=std::io::Result<&str>> {
    pub fn new(source: T) -> Self {
        Self {
            source,
        }
    }
}

impl Iterator for FlatFileReader<T> {
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