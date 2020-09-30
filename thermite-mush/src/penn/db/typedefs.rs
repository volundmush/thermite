use std::{
    rc::Rc,
    fmt::{Display, Formatter, Result, Error}
};


pub type Timestamp = isize;
pub type Money = isize;

#[derive(Eq, Clone, Debug, Hash, PartialEq)]
pub enum DbRef {
    None,
    Num(usize),
    Name(Rc<str>)
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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