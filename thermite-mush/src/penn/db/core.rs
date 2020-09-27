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
    objects::{ObjManager, Obj},
    typedefs::{DbRef, Money, Timestamp},
    props::{Property, PropertyData, PropertyManager, PropertySystem}
};

use thermite_util::{
    text::StringInterner
};

#[derive(Debug)]
pub struct GameState {
    pub interner: Rc<RefCell<StringInterner>>,
    pub objects: ObjManager,
    pub props: PropertySystem,
    pub functions: FunctionManager,
    pub commands: CommandManager,
    pub connections: HashSet<String>
}

impl Default for GameState {
    fn default() -> Self {
        let mut interner = Rc::new(RefCell::new(StringInterner::default()));

        Self {
            objects: ObjManager::new(interner.clone()),
            props: PropertySystem::new(interner.clone()),
            functions: FunctionManager::default(),
            commands: CommandManager::default(),
            connections: Default::default(),
            interner
        }
    }
}


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

impl Error for DbError {

}

impl GameState {
    pub fn load_defaults(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut f = File::open(path)?;
        let mut r = BufReader::new(f);

        let mut j: serde_json::Value = serde_json::from_reader(r)?;
        if let JV::Object(dict) = j {
            if let Some(props) = dict.get("props") {
                self.props.load_json(props);
            } else {
                return Err(DbError::new("invalid json from defaults file: props").into())
            }
        } else {
            return Err(DbError::new("invalid json from defaults file").into())
        }
        Ok(())
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