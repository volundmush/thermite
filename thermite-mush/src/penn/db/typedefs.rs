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