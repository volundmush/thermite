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
pub struct FlatLine {
    pub name: String,
    pub depth: usize,
    pub value: NodeValue,
}

impl Default for FlatLine {
    fn default() -> Self {
        Self {
            name: Default::default(),
            depth: 0,
            value: NodeValue::None
        }
    }
}

impl From<String> for FlatLine {
    fn from(src: String) -> Self {
        Self::from(src.as_str())
    }
}

impl From<&str> for FlatLine {
    fn from(src: &str) -> Self {

        let depth = count_spaces(src);
        let (spaces, data) = src.split_at(depth);
        let mut split= data.splitn(2, " ");
        let name = split.next().unwrap().to_string();
        let val = split.next().unwrap();

        if val.starts_with("\"") {
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

pub fn idx_line(flatlines: &[FlatLine], depth: usize, start: &str) -> Option<usize> {
    for (i, fline) in flatlines.iter().enumerate() {
        if fline.depth == depth && fline.name.starts_with(start) {
            return Some(i)
        }
    }
    None
}

pub fn get_idx(flatlines: &[FlatLine], depth: usize, start: &str, emsg: &str) -> Result<usize, DbError> {
    match idx_line(flatlines, depth, start) {
        Some(idx) => Ok(idx),
        None => Err(DbError::new(emsg))
    }
}

pub fn value_terminated(val: &str) -> bool {
    if !val.ends_with('"') {
        return false;
    }
    // We know for a fact that this string ends in a ".
    let (data, unused) = val.split_at(val.len()-1);
    // Scan until we encounter something that's not a \ and then count \ total...if odd, the last is
    // one escapes the " and so we are NOT terminated.

    let mut slashes: usize = 0;
    for c in data.as_bytes().iter().rev() {
        if c == &92 {
            slashes += 1;
        } else {
            break;
        }
    }
    println!("CHECKING TERMINATION: {} - {} - {}", val, slashes, slashes % 2 == 0);
    slashes % 2 == 0
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

enum SplitterState {
    Data,
    CR,
    Escaped
}

// This is designed to read PennMUSH's strange flatfile format and turn any lines which
// include 'quoted strings' into single strings.
pub struct FlatFileSplitter<T> {
    source: Bytes<T>,
    buffer: Vec<u8>,
    quoted: bool,
    state: SplitterState
}

impl<T> FlatFileSplitter<T> where T: Read {
    pub fn new(source: T) -> Self {
        Self {
            source: source.bytes(),
            buffer: Default::default(),
            quoted: false,
            state: SplitterState::Data
        }
    }
}


impl Iterator for FlatFileSplitter<T> {
    type Item = std::io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        // Consume bytes from source until we hit a newline. If we ever encounter a double-quote,
        // enter quoted mode and keep consuming until we encounter an unescaped ending double-quote
        // followed by a newline.

        loop {
            if let Some(res) = self.source.next() {
                match res {
                    Ok(c) => {

                    },
                    Err(e) => {
                        // this is some kind of io error that isn't an EOF.
                    }
                }
            } else {
                // We hit an EOF.
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
            source,
        }
    }

    fn parse_escapes(data: &str) -> String {
        let mut out = String::new();

        let mut chars = data.chars();

        loop {
            if let Some(c) = chars.next() {
                if c == '\\' {
                    if let Some(c2) = chars.next() {
                        out.push(c2);
                    } else {
                        return out
                    }
                } else {
                    out.push(c);
                }
            } else {
                return out
            }
        }
    }

    pub fn next(&mut self) -> Option<Result<FlatLine, Box<dyn Error>>> {
        let mut buffer = String::new();
        if let Some(line_res) = self.source.next() {
            match line_res {
                Ok(data) => buffer.push_str(data.as_str()),
                Err(e) => return Some(Err(Box::new(e)))
            }
        } else {
            // We have reached EOF.
            return None
        }

        let depth = count_spaces(buffer.as_str());

        if depth == 0 && buffer.starts_with("+") {
            return Some(Ok(FlatLine {
                name: buffer,
                depth,
                value: NodeValue::None
            }))
        };

        let (_, rest) = buffer.split_at(depth);

        return if let Some(idx) = rest.find(' ') {
            let (proto_name, proto_value) = rest.split_at(idx);
            let proto_name = proto_name.trim();
            let proto_value = proto_value.trim();

            let node_value = {
                if proto_value.starts_with('"') {
                    // This is a quoted string value.
                    let (_, rest) = proto_value.split_at(1);
                    let mut value = rest.to_string();

                    // We must consume chars until we reach a terminating, unescaped "
                    // if we encounter a \ we must treat it as an escape and pull in the next line.
                    while !value_terminated(value.as_str()) {
                        if let Some(add_line_res) = self.source.next() {
                            match add_line_res {
                                Ok(add_line) => {
                                    value.push('\n');
                                    value.push_str(add_line.as_str());
                                },
                                Err(e) => return Some(Err(Box::new(e)))
                            }
                        } else {
                            // We have somehow reached EOF... and we definitely should not have, here.
                            return Some(Err(Box::new(DbError::new("Unexpected EOF while processing quoted string value"))));
                        }
                    }
                    // By this point the value should be terminated. However, we don't need the trailing "
                    let (trim_value, _) = value.split_at(value.len() - 1);
                    NodeValue::Text(Self::parse_escapes(trim_value))
                } else if proto_value.starts_with('#') {
                    // This is a dbref value.
                    let (_, rest) = proto_value.split_at(1);
                    match rest.parse::<Dbref>() {
                        Ok(val) => NodeValue::Db(val),
                        Err(e) => return Some(Err(Box::new(e)))
                    }
                } else {
                    // This is any other kind of value. It will be interpreted as a number.
                    match proto_value.parse::<isize>() {
                        Ok(val) => NodeValue::Number(val),
                        Err(e) => return Some(Err(Box::new(e)))
                    }
                }
            };
            // Hooray, we have a NodeValue!
            Some(Ok(FlatLine {
                name: proto_name.to_string(),
                depth,
                value: node_value
            }))
        } else {
            // If rest doesn't have a space, it's simple - a FlatLine with no value.
            Some(Ok(FlatLine {
                depth,
                name: rest.to_string(),
                value: NodeValue::None
            }))
        }
    }
}