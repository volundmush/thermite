use std::collections::{HashSet, HashMap};
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;
use super::{
    typedefs::Dbref,
    restricts::Restriction
};

#[derive(Debug)]
pub enum FunctionAction {
    Builtin(fn() -> ()),
    User(Dbref, usize)
}

#[derive(Debug)]
pub struct Function {
    pub name: Rc<str>,
    pub restrictions: HashSet<Rc<Restriction>>,
    pub min_args: isize,
    pub max_args: isize,
    pub even_args: bool,
    pub aliases: HashSet<Rc<str>>
}

#[derive(Debug, Default)]
pub struct FunctionManager {
    pub functions: HashMap<Rc<str>, Rc<RefCell<Function>>>,
    pub alias_index: HashMap<Rc<str>, Rc<RefCell<Function>>>
}