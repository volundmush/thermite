use std::collections::{HashSet, HashMap};
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;
use super::Dbref;


pub struct AttributeFlag {
    pub name: String,
    pub aliases: HashSet<String>,
    pub letter: String,
    pub set_perms: String,
    pub reset_perms: String,
    pub see_perms: String,
    pub tree_inherit: bool,
    pub internal: bool
}

#[derive(Default)]
pub struct AttributeFlagManager {
    attribute_flags: HashMap<String, Rc<AttributeFlag>>,
    letter_index: HashMap<String, Rc<AttributeFlag>>,
    alias_index: HashMap<String, Rc<AttributeFlag>>
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
        self.letter_index.insert(rcflag.letter.clone(), rcflag.clone());
        for alias in &rcflag.aliases {
            self.alias_index.insert(alias.clone(), rcflag.clone());
        }
        self.attribute_flags.insert(rcflag.name.clone(), rcflag);

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
            name: "no_command".to_string(),
            letter: "$".to_string(),
            aliases: Default::default(),
            set_perms: "#TRUE".to_string(),
            reset_perms: "#TRUE".to_string(),
            see_perms: "#TRUE".to_string(),
            tree_inherit: true,
            internal: false
        });
        manager.add_flag(AttributeFlag {
            name: "no_inherit".to_string(),
            letter: "i".to_string(),
            aliases: Default::default(),
            set_perms: "#TRUE".to_string(),
            reset_perms: "#TRUE".to_string(),
            see_perms: "#TRUE".to_string(),
            tree_inherit: true,
            internal: false
        });
        manager.add_flag(AttributeFlag {
            name: "no_clone".to_string(),
            letter: "c".to_string(),
            aliases: Default::default(),
            set_perms: "#TRUE".to_string(),
            reset_perms: "#TRUE".to_string(),
            see_perms: "#TRUE".to_string(),
            tree_inherit: true,
            internal: false
        });

        manager.add_flag(AttributeFlag {
            name: "mortal_dark".to_string(),
            letter: "m".to_string(),
            aliases: hashset!["hidden".to_string()],
            set_perms: "#TRUE".to_string(),
            reset_perms: "#TRUE".to_string(),
            see_perms: "#TRUE".to_string(),
            tree_inherit: true,
            internal: false
        });

        manager.add_flag(AttributeFlag {
            name: "wizard".to_string(),
            letter: "w".to_string(),
            aliases: Default::default(),
            set_perms: "FLAG^ROYALTY|FLAG^WIZARD".to_string(),
            reset_perms: "FLAG^ROYALTY|FLAG^WIZARD".to_string(),
            see_perms: "#TRUE".to_string(),
            tree_inherit: true,
            internal: false
        });

        Self
    }
}

pub struct Attribute {
    pub name: String,
    pub flags: HashSet<AttributeFlag>,
    pub data: String,
    pub aliases: HashSet<String>
}


#[derive(Default)]
pub struct AttributeManager {
    pub attributes: Vec<Attribute>,
    pub name_index: HashMap<String, usize>,
    pub alias_index: HashMap<String, usize>,
    pub holders_index: HashMap<usize, HashSet<Dbref>>
}