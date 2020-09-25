use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use std::cell::{RefCell, RefMut, Ref};
use super::{
    objects::ObjType,
    core::DbError
};


#[derive(Debug, Eq, PartialEq, Hash)]
pub struct FlagPerm {
    pub name: &'static str,
    pub set_perm: &'static str,
    pub reset_perm: &'static str,
    pub system: bool,
}

#[derive(Debug)]
pub struct FlagPermManager {
    pub perms: HashMap<&'static str, Rc<FlagPerm>>
}

impl Default for FlagPermManager {
    fn default() -> Self {
        let mut manager = Self {
            perms: Default::default()
        };

        // load flag perms here...

        for perm in vec![
            FlagPerm {name: "trusted", set_perm: "#TRUE", reset_perm: "TRUE", system: false},
            FlagPerm {name: "royalty", set_perm: "FLAG^ROYALTY", reset_perm: "FLAG^ROYALTY", system: false},
            FlagPerm {name: "wizard", set_perm: "FLAG^WIZARD", reset_perm: "FLAG^WIZARD", system: false},
            FlagPerm {name: "god", set_perm: "#FALSE", reset_perm: "#FALSE", system: false},
            FlagPerm {name: "dark", set_perm: "#FALSE", reset_perm: "#FALSE", system: false},
            FlagPerm {name: "mdark", set_perm: "FLAG^WIZARD|FLAG^ROYALTY", reset_perm: "FLAG^WIZARD|FLAG^ROYALTY", system: false},
            FlagPerm {name: "odark", set_perm: "#TRUE", reset_perm: "#TRUE", system: false},
            FlagPerm {name: "internal", set_perm: "#FALSE", reset_perm: "#FALSE", system: true},
            FlagPerm {name: "log", set_perm: "FLAG^WIZARD", reset_perm: "FLAG^WIZARD", system: false},
            FlagPerm {name: "event", set_perm: "FLAG^WIZARD", reset_perm: "FLAG^WIZARD", system: false}
        ] {
            manager.perms.insert(perm.name, Rc::new(perm));
        }

        manager
    }
}

impl FlagPermManager {
    pub fn get_flag_perm(&self, name: &str) -> Option<Rc<FlagPerm>> {
        if let Some(t) = self.perms.get(name) {
            Some(t.clone())
        } else {
            None
        }
    }
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