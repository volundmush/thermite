use std::{
    collections::{HashSet, HashMap},
    cell::{RefCell, Ref, RefMut},
    rc::Rc,
    error::Error,
    fmt::{Display, Formatter},
    path::Path,
    io::{Read, BufRead, BufReader},
    fs::{File},
    cmp::max
};

use serde::Deserialize;
use serde_json;
use serde_json::Value as JV;
use serde_derive;

use legion::*;
use generational_arena::{Arena, Index};

use super::{
    functions::{FunctionManager},
    commands::{CommandManager},
    typedefs::{DbRef, Money, Timestamp, ObjType},
    schema::{
        InternString,
        Property,
        Alias,
        PropertyRelation,
        ObjectPropertyRelation,
        ObjAlias
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
pub struct PropertyManager {
    pub names: StringHolder,
    pub lockkeys: StringHolder,
    pub types: StringHolder,
    pub contents: Arena<Property>,
    pub aliases: Arena<Alias>,
    pub reltypes: StringHolder,
    pub relations: Arena<PropertyRelation>,
    pub objproprel: Arena<ObjectPropertyRelation>,
}

impl PropertyManager {
    pub fn find(&self, prop_type: usize, name_idx: usize) -> Option<Index> {
        // this will locate the row_id of a Property by its name or alias.
        if let Some((i, found)) = self.contents.iter().find(|(i, x)| x.name_match(prop_type, name_idx)) {
            return Some(i)
        }

        self.find_alias(prop_type, name_idx)
    }

    pub fn find_alias(&self, prop_type: usize, name: usize) -> Option<Index> {
        if let Some((i, found)) = self.aliases.iter().find(|(i, x) | x.name_match(prop_type, name)) {
            return Some(found.property_id)
        }
        None
    }

    pub fn find_name(&self, prop_type: usize, name: &str) -> Option<Index> {
        if let Some(name_idx) = self.names.find(name.to_uppercase().as_str()) {
            return self.find(prop_type, name_idx)
        }
        None
    }

    pub fn find_name_and_type(&self, type_name: &str, name: &str) -> Option<Index> {
        // this works similarly to find, except it takes strings, and it will not create new entries.
        if let Some(type_idx) = self.types.find(type_name.to_uppercase().as_str()) {
            if let Some(name_idx) = self.names.find(name.to_uppercase().as_str()) {
                return self.find(type_idx, name_idx)
            }
        }
        None
    }

    pub fn get_or_create_type(&mut self, type_name: &str) -> usize {
        self.types.get_or_intern(type_name.to_uppercase())
    }

    pub fn get_or_create(&mut self, type_idx: usize, name: &str) -> Index {
        let name_idx = self.names.get_or_intern(name.to_uppercase());
        if let Some(i) = self.find(type_idx, name_idx) {
            return i
        }
        let mut prop = Property::default();
        prop.name_id = name_idx;
        prop.property_type = type_idx;
        self.contents.insert(prop)
    }

    // What it says on the tin. Note that this doesn't -initialize- a property properly, only creates it.
    pub fn get_or_create_and_type(&mut self, type_name: &str, name: &str) -> Index {
        // the row id of a proptypes is its existence. these names cannot change once loaded.
        self.get_or_create(self.get_or_create_type(type_name), name)
    }

    pub fn add_alias(&mut self, prop_idx: Index, alias: &str) -> Result<(), DbError> {
        // this will error if the alias already exists on another property!
        // It will not validate if an alias name is good!
        let a_idx = self.names.get_or_intern(alias.to_uppercase());
        let type_idx = self.contents.get(prop_idx).unwrap().property_type;
        if let Some(res) = self.find_alias(type_idx, a_idx) {
            if res != prop_idx {
                return Err(DbError::new("alias already used"))
            }
            Ok(())
        } else {
            let mut new_alias = Alias::default();
            new_alias.row_id = self.propalias.len();
            new_alias.property_id = prop_idx;
            new_alias.name_id = a_idx;
            new_alias.property_type = type_idx;
            self.aliases.insert(new_alias);
            Ok(())
        }
    }

    pub fn set_letter(&mut self, prop_idx: Index, letter: &str) -> Result<(), DbError> {
        // This performs no conflict checks because of some wonkiness in how Flags work...
        let len = letter.len();
        match len {
            0 | 1 => {
                if let Some(prop) = self.contents.get_mut(prop_idx) {
                    if len == 0 {
                        prop.letter = None
                    } else {
                        prop.letter = Some(letter.chars().next().unwrap());
                    }
                    Ok(())
                } else {
                    Err(DbError::new("property not found"))
                }
            },
            _ => {
                // Reject this.
                Err(DbError::new("letters must be single characters"))
            }
        }
    }

    pub fn find_relation(&self, prop_idx: usize, relation: usize, with: usize) -> Option<usize> {
        // returns the row_id of a PropertyRelation matching the description, if exists...
    }

    pub fn add_relation(&mut self, prop_idx: usize, relation: usize, with: usize) -> Result<(), DbError> {

    }

    pub fn obj_delete(&mut self, obj: Entity, db: DbRef) -> Result<(), DbError> {
        let mut holders: Vec<Index> = Default::default();
        let mut holder_props: Vec<Index> = Default::default();

        for (i, x) in self.objproprel.iter_mut() {
            if x.object_id = obj {
                holders.push(i);
                holder_props.push(x.property_id);
            } else {
                if x.owner == db {
                    x.owner = DbRef::None
                }
            }
        }
        for idx in found {
            self.objproprel.remove(idx)
        }

        for i in holder_props {
            if let Some(x) = self.contents.get_mut(i) {
                x.owner = DbRef::None;
                let _ = x.holders.remove(&obj);
            }
        }



        Ok(())
    }
}

#[derive(Default)]
pub struct DbNumManager {
    pub greatest: usize,
    pub available: Vec<usize>,
}

impl DbNumManager {
    pub fn init(&mut self, obj: &HashMap<DbRef, Entity>) {
        for (i, o) in obj.iter().filter(|(i, o)| i.dbref.is_num()) {
            self.greatest = cmp(self.greatest, i.dbref.to_num());
        }
    }

    pub fn scan_recycle(&mut self, dbrefs: &HashMap<DbRef, Entity>) {
        for i in 0..self.greatest {
            let db = DbRef::from(i);
            if !dbrefs.contains_key(&db) {
                self.available.push(i);
                if self.available.len() > 50 {
                    break
                }
            }
        }
    }

    pub fn first_available(&mut self, dbrefs: &HashMap<DbRef, Entity>) -> DbRef {
        if let Some(avail) = self.available.pop() {
            DbRef::from(avail)
        } else {
            self.scan_recycle(&dbrefs);
            if let Some(avail) = self.available.pop() {
                DbRef::from(avail)
            } else {
                DbRef::from(self.greatest + 1)
            }
        }
    }

    pub fn create(&mut self, dbrefs: &HashMap<DbRef, Entity>, choice: DbRef) -> Result<DbRef, DbError> {
        // given a Dbref, return a valid DbRef to use or Error. If given a SPECIFIC DbRef, it is
        // available if it is not taken and is <= Greatest. Given a DbRef::None, pick the first available.

        if dbrefs.contains_key(&choice) {
            return Err(DbError::new("dbref already in use"))
        }

        match choice {
            DbRef::None => Ok(self.first_available(&dbrefs)),
            DbRef::Num(n) => {
                if n > self.greatest {
                    return Err(DbError::new("dbref higher than highest used"))
                } else {
                    Ok(choice)
                }
            },
            DbRef::Name(usize) => Ok(choice)
        }
    }
}


#[derive(Debug, Default)]
pub struct GameState {
    pub names: StringHolder,
    pub uppers: StringHolder,
    pub props: PropertyManager,
    pub reltypes: StringHolder,
    pub dbnums: DbNumManager,
    pub dbrefs: HashMap<DbRef, Entity>,
    pub objalias: Arena<ObjAlias>
}

impl GameState {

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
                let type_idx = self.props.get_or_create_type(prop_type_name);
                if let JV::Object(props) = v {
                    for (propname, def) in props {
                        let mut prop_idx = self.props.get_or_create(type_idx, propname);
                        if let JV::Object(fields) = def {

                            // Aliases
                            if let Some(alias_j) = fields.get("aliases") {
                                if let JV::Array(alias_j_l) = alias_j {
                                    for ali_v in alias_j_l {
                                        if let JV::String(alias) = ali_v {
                                            self.props.add_alias(prop_idx, alias)?;
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
                                            let mut prop = self.props.properties.get_mut(prop_idx).unwrap();
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

    pub fn obj_create_valid(&mut self, db: DbRef, name: &str, type_idx: Index) -> Result<DbRef, DbError> {
        // Performs the deepest level of object creation - name conflict checking, type checking, and DbRef availability.
        // if DbRef is DbRef::None, will automatically choose a DbRef.
        // However, nothing is written to database / game state. This merely validates.
        if type_idx == self.props.get_or_create_and_type("OBJ_TYPE", "PLAYER") {
            // Must do a name validation on possible Players.
        }
        // Even if names check out, the given DbRef must be figured out or chosen. If so, this is a
        // valid combination...
        Ok(self.dbnums.create(&self.dbrefs, db)?)
    }

    pub fn obj_delete(&mut self, db: DbRef) -> Result<(), DbError> {
        if let Some(idx) = self.dbrefs.remove(&db) {
            self.props.obj_delete(idx, db)?;

            let found = self.objalias.iter().filter(|(i, x)| x.object_id == idx).map(|(i, x)| i).collect_vec();
            for i in found {
                self.objalias.remove(i);
            }

            Ok(())
        } else {
            return Err(DbError::new("object does not exist"))
        }
    }
}


pub struct GameUniverse {
    pub world: World,
    pub resources: Resources,
    pub schedule: Schedule
}

impl Default for GameUniverse {
    fn default() -> Self {
        let mut schedule = Schedule::builder().build();

        let mut uni = Self {
            world: Default::default(),
            resources: Default::default(),
            schedule
        };


        uni
    }
}