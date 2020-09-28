use std::{
    rc::Rc,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    cell::{RefCell}
};

use super::{
    core::DbError,
    objects::Obj,
    typedefs::DbRef
};

use thermite_util::{
    text::{StringInterner}
};

use serde_json::Value;

#[derive(Debug)]
pub struct ObjProperty {
    pub creator: DbRef,
    pub owner: DbRef,
    pub props: HashSet<Rc<Property>>,
    pub value: String
}

#[derive(Debug, Default)]
pub struct PropertyData {
    pub aliases: HashSet<usize>,
    pub set_perms: usize,
    pub reset_perms: usize,
    pub see_perms: usize,
    pub tree_inherit: bool,
    pub system: bool,
    pub internal: bool,
    pub letter: Option<char>,
    pub allowed_types: HashSet<Rc<Property>>,
    pub flags: HashSet<Rc<Property>>,
    pub holders: HashSet<Rc<Obj>>,
    pub creator: DbRef,
    pub data: String
}

#[derive(Debug)]
pub struct Property {
    pub name: usize,
    pub data: RefCell<PropertyData>,
    pub manager: Rc<RefCell<PropertyManager>>
}

impl Property {
    pub fn new(name: usize, manager: Rc<RefCell<PropertyManager>>) -> Self {
        Self {
            name,
            data: RefCell::new(PropertyData::default()),
            manager
        }
    }

    pub fn held_by(&self, obj: &Rc<Obj>) -> bool {
        self.data.borrow().holders.contains(obj)
    }

    pub fn aliases_vec(&self) -> Vec<usize> {
        let v: Vec<usize> = self.data.borrow().aliases.iter().map(|x| *x).collect();
        v
    }

    pub fn add_alias(&self, alias: &str) -> Result<(), DbError>
    {
        let _ = self.data.borrow_mut().aliases.insert(alias);
        Ok(())
    }

    pub fn remove_alias(&self, alias: &str) -> Result<(), DbError> {
        let _ = self.data.borrow_mut().aliases.remove(alias);
        Ok(())
    }

    pub fn set_letter(&self, letter: &str) -> Result<(), DbError> {
        if letter.len() > 1 {
            return Err(DbError::new("Cannot set letter to more than one character!"))
        }
        if letter.len() == 0 {
            // this will remove the letter.
            if let Some(c) = self.data.borrow().letter {
                let _ = self.manager.borrow_mut().letters.remove(&c)
            }
            self.data.borrow_mut().letter = None;
        }

        let c = letter.chars().next().unwrap();
        if self.manager.borrow().letters.contains_key(&c) {
            return Err(DbError::new("That would conflict with another property"))
        }

        self.manager.borrow_mut().set_letter(self.name, c);
        self.data.borrow_mut().letter = Some(c);
        Ok(())
    }

}

impl PartialEq for Property {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Property {}


// This is for managing indexable things which are unique-by-case-insensitive-immutable-names
#[derive(Default, Debug)]
pub struct PropertyManager {
    pub name: String,
    pub contents: HashMap<usize, Rc<Property>>,
    pub aliases: HashMap<usize, Rc<Property>>,
    pub letters: HashMap<char, Rc<Property>>,
    pub interner: Rc<RefCell<StringInterner>>
}

impl PropertyManager {

    pub fn new(name: String, interner: Rc<RefCell<StringInterner>>) -> Self {
        Self {
            name,
            contents: Default::default(),
            aliases: Default::default(),
            letters: Default::default(),
            interner
        }
    }

    pub fn valid_name(&self, name: &str) -> bool
    {
        true
    }

    pub fn get_or_create(&mut self, name: &str, manager: Rc<RefCell<PropertyManager>>) -> Rc<Property> {
        // this function assumes that an attribute name has already been validated.
        if let Some(res) = self.get_property(name, false, false) {
            return res
        }
        let idx = self.interner.borrow_mut().get_or_intern(name.to_uppercase().as_str());
        let new_prop = Rc::new(Property::new(idx, manager));
        self.contents.insert(idx, new_prop.clone());
        return new_prop
    }

    fn find_idx(&self, idx: &HashMap<usize, Rc<Property>>, name: &str) -> Option<Rc<Property>> {
        let mut idx_vec: Vec<usize> = idx.keys().map(|x| *x).collect();
        if let Some(i) = self.interner.borrow().exact_match(idx_vec.as_slice(), name) {
            if let Some(res) = idx.get(&i) {
                return Some(res.clone())
            }
        }
        None
    }

    pub fn get_property(&self, name: &str, check_alias: bool, check_letter: bool) -> Option<Rc<Property>> {
        if name.len() == 1 && check_letter {
            let c = name.chars().next().unwrap();
            if let Some(res) = self.letters.get(&c) {
                return Some(res.clone())
            }
        }

        let upper = name.to_uppercase();
        let name_match = self.find_idx(&self.contents, upper.as_str());
        if name_match.is_some() {
            return name_match
        }
        if check_alias {
            let alias_match = self.find_idx(&self.aliases, upper.as_str());
            if alias_match.is_some() {
                return alias_match
            }
        }
        None
    }

    pub fn has_prop(&self, obj: &Rc<Obj>, name: &str, check_alias: bool, check_letter: bool) -> bool {
        if let Some(res) = self.get_property(name, check_alias, check_letter) {
            res.held_by(obj)
        } else {
            return false
        }
    }

    pub fn set_letter(&mut self, idx: usize, letter: char) {
        if let Some(r) = self.contents.get(&idx) {
            self.letters.insert(letter, r.clone())
        }
    }
}

#[derive(Debug)]
pub struct PropertySystem {
    pub managers: HashMap<String, Rc<RefCell<PropertyManager>>>,
    pub interner: Rc<RefCell<StringInterner>>
}

impl Default for PropertySystem {
    fn default() -> Self {
        Self {
            managers: Default::default(),
            interner: Rc::new(RefCell::new(StringInterner::default()))
        }
    }
}

impl PropertySystem {
    pub fn new(interner: Rc<RefCell<StringInterner>>) -> Self {
        Self {
            interner,
            managers: Default::default()
        }
    }

    pub fn load_json(&mut self, data: &Value) -> Result<(), DbError> {

        if let Value::Object(sections) = data {
            for (k, v) in sections {
                let mut manager = self.get_or_create(k);

                if let Value::Object(props) = v {
                    for (name, def) in props {
                        let mut prop = manager.borrow_mut().get_or_create(name, manager.clone());
                        if let Value::Object(fields) = def {

                            // Aliases
                            if let Some(alias_j) = fields.get("aliases") {
                                if let Value::Array(alias_j_l) = alias_j {
                                    for ali_v in alias_j_l {
                                        if let Value::String(alias) = ali_v {
                                            prop.add_alias(alias)?;
                                        } else {
                                            return Err(DbError::new("alias data must be an array of strings"));
                                        }
                                    }
                                } else {
                                    return Err(DbError::new("alias data must be an array of strings"));
                                }
                            }

                            // Letter
                            if let Some(letter_j) = fields.get("letter") {
                                if let Value::String(letter) = letter_j {
                                    if letter.len() > 0 {
                                        prop.set_letter(letter)?;
                                    }
                                } else {
                                    return Err(DbError::new("letter data must be a string containing one character"));
                                }
                            }




                        } else {
                            return Err(DbError::new("no json data found for fields of prop"));
                        }
                    }
                }

            }
        } else {
            return Err(DbError::new("no json data found for props"));
        }


        Ok(())
    }

    pub fn get_or_create(&mut self, name: &str) -> Rc<RefCell<PropertyManager>> {
        let upper = name.to_uppercase();
        if let Some(res) = self.managers.get(&upper) {
            return res.clone()
        }
        let new_manager = Rc::new(RefCell::new(PropertyManager::new(upper.clone(), self.interner.clone())));
        self.managers.insert(upper, new_manager.clone());
        return new_manager
    }

    pub fn get_property(&self, manager: &str, name: &str, check_alias: bool, check_letter: bool) -> Option<Rc<Property>> {
        let upper = manager.to_uppercase();
        if let Some(manager) = self.managers.get(&upper) {
            return manager.borrow().get_property(name, check_alias, check_letter)
        } else {
            None
        }
    }

    pub fn obj_has_property(&self, obj: &Rc<Obj>, manager: &str, name: &str, check_alias: bool, check_letter: bool) -> bool {
        let upper = manager.to_uppercase();

        if let Some(res) = self.get_property(manager, name, check_alias, check_letter) {
            res.held_by(obj)
        } else {
            return false
        }
    }

}