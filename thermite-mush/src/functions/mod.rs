use std::collections::{HashSet, HashMap};
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;
use crate::{
    softcode::typedefs::DbRef,
};

#[derive(Debug)]
pub enum FunctionAction {
    Builtin(fn() -> ()),
    User(DbRef, usize)
}

#[derive(Debug)]
pub struct Function {
    pub min_args: isize,
    pub max_args: isize,
    pub even_args: bool,
    pub action: FunctionAction
}

#[derive(Debug, Default)]
pub struct FunctionManager {
    pub functions: HashMap<usize, Function>,
}