use std::{
    rc::Rc,
    fmt::{Display, Formatter, Error},
    convert::{TryFrom}
};

use super::{
    core::{DbError}
};

pub type Timestamp = isize;
pub type Money = isize;

// the name is String-Interned.
#[derive(Eq, Clone, Debug, Hash, PartialEq)]
pub enum DbRef {
    None,
    Num(usize),
    Name(usize)
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



pub enum ObjType {
    Player,
    Room,
    Thing,
    Exit
}

impl TryFrom<isize> for ObjType {
    type Error = DbError;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        match value {
            8 => Ok(Self::Player),
            4 => Ok(Self::Exit),
            2 => Ok(Self::Thing),
            1 => Ok(Self::Room),
            _ => Err(DbError::new("invalid serialization for ObjectType"))
        }
    }
}

impl TryFrom<char> for ObjType {
    type Error = DbError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_uppercase() {
            'P' => Ok(Self::Player),
            'E' => Ok(Self::Exit),
            'T' => Ok(Self::Thing),
            'R' => Ok(Self::Room),
            _ => Err(DbError::new("invalid serialization for ObjectType"))
        }
    }
}

impl TryFrom<String> for ObjType {
    type Error = DbError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != 1 {
            Err(DbError::new("invalid serialization for ObjectType"))
        } else {
            Self::try_from(value.chars().next().unwrap())
        }
    }
}