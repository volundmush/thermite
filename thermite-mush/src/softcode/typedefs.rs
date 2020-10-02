use std::{
    rc::Rc,
    fmt::{Display, Formatter},
    convert::{TryFrom},
    error::Error
};


pub type Timestamp = isize;
pub type Money = isize;

// the name is String-Interned.
#[derive(Eq, Clone, Debug, Hash, PartialEq)]
pub enum DbRef {
    None,
    Num(usize),
    Name(String)
}

impl DbRef {
    pub fn to_string(&self) -> String {
        match self {
            Self::None => "#-1".to_string(),
            Self::Num(n) => {
                format!("#{}", n)
            },
            Self::Name(r) => r.to_string()
        }
    }

    pub fn is_num(&self) -> bool {
        match self {
            Self::None => false,
            Self::Name(r) => false,
            Self::Num(n) => true
        }
    }

    pub fn is_name(&self) -> bool {
        match self {
            Self::None => false,
            Self::Name(r) => true,
            Self::Num(n) => false
        }
    }

    pub fn to_num(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Name(r) => 0,
            Self::Num(n) => *n
        }
    }
}

impl Display for DbRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for DbRef {
    fn default() -> Self {
        Self::None
    }
}

impl From<usize> for DbRef {
    fn from(src: usize) -> Self {
        DbRef::Num(src)
    }
}

impl From<String> for DbRef {
    fn from(src: String) -> Self {DbRef::Name(src.to_uppercase())}
}


#[derive(Debug)]
pub struct DbError {
    data: String
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!( f, "{}", self.data)
    }
}

impl DbError {
    pub fn new(src: &str) -> Self {
        Self {
            data: src.to_string()
        }
    }
}

impl From<&str> for DbError {
    fn from(src: &str) -> Self {
        Self {
            data: src.to_string()
        }
    }
}

impl From<String> for DbError {
    fn from(src: String) -> Self {
        Self {
            data: src
        }
    }
}
impl Error for DbError {}


pub enum DbAddr {
    None,
    Ref(DbRef),
    Objid(usize, Timestamp)
}

// #TODO: Implement TryFrom<String> for DbAddr