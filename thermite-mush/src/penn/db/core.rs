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


    pub fn find_property(&self, prop_type: usize, name: usize) -> Option<usize> {
        // this will locate the row_id of a Property by its name or alias.
        if let Some(found) = self.properties.iter().find(|x| x.name_match(prop_type, name)) {
            return Some(found.row_id)
        }

        if let Some(found) = self.propalias.iter().find(|x| x.name_match(prop_type, name)) {
            return Some(found.property_id)
        }
        None
    }

    pub fn find_property_name(&self, type_name: &str, name: &str) -> Option<usize> {
        // this works similarly to find, except it takes strings, and it will not create new entries.
        if let Some(type_idx) = self.proptypes.find(type_name.to_uppercase().as_str()) {
            if let Some(name_idx) = self.propnames.find(name.to_uppercase().as_str()) {
                return self.find_property(type_idx, name_idx)
            }
        }
        None
    }

    // What it says on the tin. Note that this doesn't -initialize- a property properly, only creates it.
    pub fn get_or_create_property(&mut self, type_name: &str, name: &str) -> usize {
        // the row id of a proptypes is its existence. these names cannot change once loaded.
        let type_idx = self.proptypes.get_or_intern(type_name.to_uppercase());
        let name_idx = self.propnames.get_or_intern(name.to_uppercase());

        if let Some(i) = self.find_property(type_idx, name_idx) {
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

    pub fn load_defaults(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut f = File::open(path)?;
        let mut r = BufReader::new(f);

        let mut j: serde_json::Value = serde_json::from_reader(r)?;
        if let JV::Object(dict) = j {
            if let Some(props) = dict.get("props") {
                self.load_props(props)?


            } else {
                return Err(DbError::new("invalid json from defaults file: props").into())
            }
        } else {
            return Err(DbError::new("invalid json from defaults file").into())
        }
        Ok(())
    }

    pub fn load_props(&mut self, data: JV) -> Result<(), Box<dyn Error>> {

    }


    pub fn get_obj(&self, db: DbRef) -> Result<Rc<Obj>, DbError> {
        if let Some(r) = self.objects.objects.get(&db) {
            Ok(r.clone())
        } else {
            Err(DbError::new("object not found"))
        }
    }

    pub fn get_bitlevel(&self, obj: &Rc<Obj>) -> usize {
        if let DbRef::Num(i) = obj.db {
            if i == 1 {
                return 7
            }
        }

        if self.props.obj_has_property(obj, "flag", "WIZARD", false, false) {
            return 6
        }
        if self.props.obj_has_property(obj, "flag", "ROYALTY", false, false) {
            return 5
        }
        if self.props.obj_has_property(obj, "power", "GUEST", false, false) {
            return 0
        }
        1
    }

    pub fn obj_controls(&self, obj: &Rc<Obj>, vic: &Rc<Obj>) -> bool {
        if self.props.obj_has_property(obj, "power", "GUEST", false, false) {
            return false
        }
        if obj == vic {
            return true
        }
        let v_bitlevel = self.get_bitlevel(vic);
        if self.get_bitlevel(vic) >= 7 {
            return false
        }
        let o_bitlevel = self.get_bitlevel(obj);
        if o_bitlevel >= 5 && (o_bitlevel > v_bitlevel) {
            return true
        }
        if self.props.obj_has_property(obj, "flag", "MISTRUST", false, false) {
            return false
        }
        let same_owner = obj.data.borrow().owner == vic.data.borrow().owner;
        if same_owner {
            if !self.props.obj_has_property(vic, "flag", "TRUST", false, false) {
                return true
            }
            if self.props.obj_has_property(obj, "flag", "TRUST", false, false) {
                return true
            }
        }
        let ply = self.props.get_property("obj_type", "PLAYER", false, false).unwrap();
        if self.props.obj_has_property(vic, "flag", "TRUST", false, false) || vic.obj_type == ply {
            return false
        }

        false
    }

}