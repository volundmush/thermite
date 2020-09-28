use std::{
    collections::{HashSet, HashMap},
    cell::{RefCell, Ref, RefMut},
    rc::Rc,
    error::Error,
    fmt::{Display, Formatter},
    path::Path,
    io::{Read, BufRead, BufReader},
    fs::{File}
};

use serde::Deserialize;
use serde_json;
use serde_json::Value as JV;
use serde_derive;

use super::{
    functions::{FunctionManager},
    commands::{CommandManager},
    typedefs::{DbRef, Money, Timestamp},
    schema::{
        InternString,
        Property,
        Alias,
        PropertyRelation,
        Object,
        ObjectMap,
        ObjectPropertyRelation,
        ObjectDataRelation
    }
};

#[derive(Debug)]
pub struct DbError {
    data: String
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!( f, "{}", self.data)
    }
}

impl DbError {
    pub fn new(src: &str) -> Self {
        Self {
            data: src.to_string()
        }
    }
}

impl From<&str> for DbError {
    fn from(src: &str) -> Self {
        Self {
            data: src.to_string()
        }
    }
}

impl From<String> for DbError {
    fn from(src: String) -> Self {
        Self {
            data: src
        }
    }
}
impl Error for DbError {}

#[derive(Debug, Default)]
pub struct StringHolder {
    pub contents: Vec<InternString>
}

impl StringHolder {
    pub fn get(&self, idx: usize) -> String {
        // this will PANIC if the index doesn't exist...
        self.contents.get(idx).unwrap().text.clone()
    }

    pub fn find(&self, data: &str) -> Option<usize> {
        if let Some(res) = self.contents.iter().find(|x| x.text.eq(data)) {
            return Some(res.row_id)
        }
        None
    }

    // WARNING: only call when you're sure this string isn't already in the system!
    pub fn intern(&mut self, data: String) -> usize {
        let idx = self.contents.len();
        self.contents.push(InternString {
            row_id: idx,
            text: data
        });
        idx
    }

    pub fn get_or_intern(&mut self, data: String) -> usize {
        if let Some(res) = self.find(&data) {
            return res
        } else {
            self.intern(data)
        }
    }
}


#[derive(Debug, Default)]
pub struct GameState {
    pub names: StringHolder,
    pub uppers: StringHolder,
    pub lockkeys: StringHolder,
    pub proptypes: StringHolder,
    pub propnames: StringHolder,
    pub properties: Vec<Property>,
    pub propalias: Vec<Alias>,
    pub proprelations: Vec<PropertyRelation>,
    pub reltypes: StringHolder,
    pub objects: Vec<Object>,
    pub dbrefs: Vec<ObjectMap>,
    pub objproprel: Vec<ObjectPropertyRelation>,
    pub objdatrel: Vec<ObjectDataRelation>,
    pub objalias: Vec<Alias>
}

impl GameState {


    pub fn property_find(&self, prop_type: usize, name_idx: usize) -> Option<usize> {
        // this will locate the row_id of a Property by its name or alias.
        if let Some(found) = self.properties.iter().find(|x| x.name_match(prop_type, name_idx)) {
            return Some(found.row_id)
        }

        self.property_find_alias(prop_type, name_idx)
    }

    pub fn property_find_alias(&self, prop_type: usize, name: usize) -> Option<usize> {
        if let Some(found) = self.propalias.iter().find(|x| x.name_match(prop_type, name)) {
            return Some(found.property_id)
        }
        None
    }

    pub fn property_find_name(&self, prop_type: usize, name: &str) -> Option<usize> {
        if let Some(name_idx) = self.propnames.find(name.to_uppercase().as_str()) {
            return self.property_find(prop_type, name_idx)
        }
        None
    }

    pub fn property_find_name_and_type(&self, type_name: &str, name: &str) -> Option<usize> {
        // this works similarly to find, except it takes strings, and it will not create new entries.
        if let Some(type_idx) = self.proptypes.find(type_name.to_uppercase().as_str()) {
            if let Some(name_idx) = self.propnames.find(name.to_uppercase().as_str()) {
                return self.property_find(type_idx, name_idx)
            }
        }
        None
    }

    pub fn property_get_or_create_type(&mut self, type_name: &str) -> usize {
        self.proptypes.get_or_intern(type_name.to_uppercase())
    }

    pub fn property_get_or_create(&mut self, type_idx: usize, name: &str) -> usize {
        let name_idx = self.propnames.get_or_intern(name.to_uppercase());
        if let Some(i) = self.property_find(type_idx, name_idx) {
            return i
        }
        let mut prop = Property::default();
        let prop_idx = self.properties.len();
        prop.name_id = name_idx;
        prop.property_type_id = type_idx;
        prop.row_id = prop_idx;
        self.properties.push(prop);
        prop_idx
    }

    // What it says on the tin. Note that this doesn't -initialize- a property properly, only creates it.
    pub fn property_get_or_create_and_type(&mut self, type_name: &str, name: &str) -> usize {
        // the row id of a proptypes is its existence. these names cannot change once loaded.
        let type_idx = self.property_get_or_create_type(type_name);
        self.property_get_or_create(type_idx, name)
    }

    pub fn property_add_alias(&mut self, prop_idx: usize, alias: &str) -> Result<(), DbError> {
        // this will error if the alias already exists! It will not validate if an alias name is good!
        let a_idx = self.propnames.get_or_intern(alias.to_uppercase());
        let type_idx = self.properties.get(prop_idx).unwrap().property_type_id;
        if let Some(res) = self.property_find_alias(type_idx, a_idx) {
            // return Err(DbError::new("alias already used"))
            Ok(())
        } else {
            let mut new_alias = Alias::default();
            new_alias.row_id = self.propalias.len();
            new_alias.property_id = prop_idx;
            new_alias.name_id = a_idx;
            new_alias.property_type_id = type_idx;
            self.propalias.push(new_alias);
            Ok(())
        }
    }


    pub fn load_defaults(&mut self, data: &JV) -> Result<(), DbError> {
        if let JV::Object(dict) = data {
            if let Some(props) = dict.get("props") {
                self.load_props(props)?;


            } else {
                return Err(DbError::new("invalid json from defaults file: props"))
            }
        } else {
            return Err(DbError::new("invalid json from defaults file"))
        }
        Ok(())
    }

    pub fn load_props(&mut self, data: &JV) -> Result<(), DbError> {
        if let JV::Object(sections) = data {
            for (prop_type_name, v) in sections {
                let type_idx = self.property_get_or_create_type(prop_type_name);
                if let JV::Object(props) = v {
                    for (propname, def) in props {
                        let mut prop_idx = self.property_get_or_create(type_idx, propname);
                        if let JV::Object(fields) = def {

                            // Aliases
                            if let Some(alias_j) = fields.get("aliases") {
                                if let JV::Array(alias_j_l) = alias_j {
                                    for ali_v in alias_j_l {
                                        if let JV::String(alias) = ali_v {
                                            self.property_add_alias(prop_idx, alias)?;
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
                                if let JV::String(letter) = letter_j {
                                    if letter.len() > 0 {
                                        if letter.len() == 1 {
                                            let letter = letter.chars().next().unwrap();
                                            let mut prop = self.properties.get_mut(prop_idx).unwrap();
                                            prop.letter = Some(letter);
                                        }
                                    }
                                } else {
                                    return Err(DbError::new("letter data must be a string containing one character"));
                                }
                            }

                            // Perms Section
                            if let Some(see) = fields.get("see_perms") {
                                if let JV::String(p) = see {
                                    let lock_idx = self.lockkeys.get_or_intern(p.to_string());
                                    let mut prop = self.properties.get_mut(prop_idx).unwrap();
                                    prop.see_perms = lock_idx;
                                } else {
                                    return Err(DbError::new("perm data must be a lock string"));
                                }
                            }

                            if let Some(see) = fields.get("set_perms") {
                                if let JV::String(p) = see {
                                    let lock_idx = self.lockkeys.get_or_intern(p.to_string());
                                    let mut prop = self.properties.get_mut(prop_idx).unwrap();
                                    prop.set_perms = lock_idx;
                                } else {
                                    return Err(DbError::new("perm data must be a lock string"));
                                }
                            }

                            if let Some(see) = fields.get("reset_perms") {
                                if let JV::String(p) = see {
                                    let lock_idx = self.lockkeys.get_or_intern(p.to_string());
                                    let mut prop = self.properties.get_mut(prop_idx).unwrap();
                                    prop.reset_perms = lock_idx;
                                } else {
                                    return Err(DbError::new("perm data must be a lock string"));
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
}