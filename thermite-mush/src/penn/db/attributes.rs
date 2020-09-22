use std::collections::{HashSet, HashMap};
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;
use super::Dbref;

pub struct AttributeFlag {
    pub name: &'static str,
    pub aliases: HashSet<&'static str>,
    pub letter: &'static str,
    pub set_perms: &'static str,
    pub reset_perms: &'static str,
    pub see_perms: &'static str,
    pub tree_inherit: bool,
    pub internal: bool
}

#[derive(Default)]
pub struct AttributeFlagManager {
    attribute_flags: HashMap<&'static str, Rc<AttributeFlag>>,
    letter_index: HashMap<&'static str, Rc<AttributeFlag>>,
    alias_index: HashMap<&'static str, Rc<AttributeFlag>>
}

impl AttributeFlagManager {
    fn add_flag(&mut self, flag: AttributeFlag) {
        // First, make sure there are no conflicts.
        // This will panic if there are any conflicts. Be careful.
        if self.attribute_flags.contains_key(&flag.name) {
            panic!("Attempted to create a duplicate AttributeFlag!")
        }
        if self.letter_index.contains_key(&flag.letter) {
            panic!("Attempted to create a duplicate AttributeFlag!")
        }

        for alias in &flag.aliases {
            if self.alias_index.contains_key(alias) {
                panic!("Attempted to create a duplicate AttributeFlag!")
            }
        }

        let rcflag = Rc::new(flag);
        self.letter_index.insert(rcflag.letter, rcflag.clone());
        for alias in &rcflag.aliases {
            self.alias_index.insert(alias, rcflag.clone());
        }
        self.attribute_flags.insert(rcflag.name, rcflag);

    }
}

impl Default for AttributeFlagManager {
    fn default() -> Self {
        let mut manager = Self {
            attribute_flags: Default::default(),
            letter_index: Default::default(),
            alias_index: Default::default(),
        };
        manager.add_flag(AttributeFlag {
            name: "no_command",
            letter: "$",
            aliases: Default::default(),
            set_perms: "#TRUE",
            reset_perms: "#TRUE",
            see_perms: "#TRUE",
            tree_inherit: true,
            internal: false
        });
        manager.add_flag(AttributeFlag {
            name: "no_inherit",
            letter: "i",
            aliases: Default::default(),
            set_perms: "#TRUE",
            reset_perms: "#TRUE",
            see_perms: "#TRUE",
            tree_inherit: true,
            internal: false
        });
        manager.add_flag(AttributeFlag {
            name: "no_clone",
            letter: "c",
            aliases: Default::default(),
            set_perms: "#TRUE",
            reset_perms: "#TRUE",
            see_perms: "#TRUE",
            tree_inherit: true,
            internal: false
        });

        manager.add_flag(AttributeFlag {
            name: "mortal_dark",
            letter: "m",
            aliases: hashset!["hidden"],
            set_perms: "#TRUE",
            reset_perms: "#TRUE",
            see_perms: "#TRUE",
            tree_inherit: true,
            internal: false
        });

        manager.add_flag(AttributeFlag {
            name: "wizard",
            letter: "w",
            aliases: Default::default(),
            set_perms: "FLAG^ROYALTY|FLAG^WIZARD",
            reset_perms: "FLAG^ROYALTY|FLAG^WIZARD",
            see_perms: "#TRUE",
            tree_inherit: true,
            internal: false
        });

        Self
    }
}

pub struct Attribute {
    pub name: Rc<str>,
    pub flags: HashSet<Rc<AttributeFlag>>,
    pub data: String,
    pub aliases: HashSet<Rc<str>>,
    pub internal: bool
}


#[derive(Default)]
pub struct AttributeManager {
    pub attributes: HashMap<Rc<str>, Rc<RefCell<Attribute>>>,
    pub alias_index: HashMap<Rc<str>, Rc<RefCell<Attribute>>>,
    pub flags: AttributeFlagManager
}

impl AttributeManager {
    pub fn add_attribute(&mut self, attr: Attribute) {

    }
}

impl Default for AttributeManager {
    fn default() -> Self {
        let mut manager = Self {
            attributes: Default::default(),
            alias_index: Default::default(),
            flags: Default::default()
        };

        // add default attributes here....

    }
}