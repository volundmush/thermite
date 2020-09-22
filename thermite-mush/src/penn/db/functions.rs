use std::collections::{HashSet, HashMap};
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;
use super::Dbref;

pub enum FunctionRestriction2 {
    God,
    Wizard,
    Admin,
    NoGagged,
    NoFixed,
    NoGuest,
    Nobody,
    Logname,
    Logargs,
    Noparse,
    Localize,
    Userfn,
    Nosidefx,
    Deprecated,
    NoPlayer
}

pub struct FunctionRestriction {
    pub name: &'static str,
    pub letter: &'static str
}

pub struct FunctionRestrictionManager {
    pub restrictions: HashMap<&'static str, Rc<FunctionRestriction>>,
    pub letter_index: HashMap<&'static str, Rc<FunctionRestriction>>
}

impl FunctionRestrictionManager {
    fn add_restriction(&mut self, restrict: FunctionRestriction) {

    }
}

impl Default for FunctionRestrictionManager {
    fn default() -> Self {
        let mut manager = Self {
            restrictions: Default::default(),
            letter_index: Default::default()
        };

        // load restrictions here

        Self
    }
}

pub enum FunctionAction {
    Builtin(fn() -> ()),
    User(Dbref, usize)
}

pub struct Function {
    pub name: String,
    pub restrictions: HashSet<FunctionRestriction>,
    pub min_args: isize,
    pub max_args: isize,
    pub even_args: bool,
    pub aliases: HashSet<String>
}

#[derive(Default)]
pub struct FunctionManager {
    pub functions: Vec<Function>,
    pub func_index: HashMap<String, usize>
}