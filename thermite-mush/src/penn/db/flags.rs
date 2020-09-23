use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use std::cell::{RefCell, RefMut, Ref};
use super::{
    objects::ObjType,
    core::DbError
};


#[derive(Debug)]
pub struct FlagPerm {
    pub name: &'static str,
    pub set_perm: &'static str,
    pub reset_perm: &'static str
}

#[derive(Debug)]
pub struct Flag {
    pub name: Rc<str>,
    pub letter: Rc<str>,
    pub obj_types: HashSet<Rc<ObjType>>,
    pub perms: HashSet<Rc<FlagPerm>>,
    pub negate_perms: HashSet<Rc<FlagPerm>>,
    pub aliases: HashSet<Rc<str>>,
    pub holders: HashSet<Rc<RefCell<ObjType>>>
}

#[derive(Default, Debug)]
pub struct FlagManager {
    pub flags: HashMap<Rc<str>, Rc<RefCell<Flag>>>,
    pub letter_index: HashMap<Rc<str>, Rc<RefCell<Flag>>>,
    pub alias_index: HashMap<Rc<str>, Rc<RefCell<Flag>>>,
}

impl FlagManager {
    pub fn load(&mut self, flags: Vec<Flag>) -> Result<(), DbError> {
        for flag in flags {
            self.add_flag(flag)?;
        }
        Ok(())
    }

    pub fn add_flag(&mut self, flag: Flag) -> Result<(), DbError> {
        let name = flag.name.clone();
        let letter = flag.letter.clone();

        let aliases = flag.aliases.clone();

        if self.letter_index.contains_key(&letter) {
            return Err(DbError::new("A flag with this letter already exists."))
        }

        for alias in &aliases {
            if self.alias_index.contains_key(alias) {
                return Err(DbError::new("A flag with this name or alias already exists."))
            }
        }
        // All verifications have passed - perform the add.
        let mut rfc = Rc::new(RefCell::new(flag));
        self.letter_index.insert(letter, rfc.clone());
        for alias in &aliases {
            self.alias_index.insert(alias.clone(), rfc.clone());
        };
        self.flags.insert(name, rfc);

        Ok(())
    }
}