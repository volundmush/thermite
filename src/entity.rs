use std::time::{SystemTime, UNIX_EPOCH};
use nalgebra::base::Vector3;
use std::collections::{HashMap, HashSet};
use std::mem::replace;

#[derive(Clone)]
pub struct EntityLocation {
    // An Entity's Location is a combination of the ID of its location, and
    // its coordinates as a Vector3.
    pub location: usize,
    // Probably want to use something other than a usize here... not sure yet.
    pub coordinates: Vector3<usize>
}

#[derive(Clone)]
pub struct EntityName {
    pub name: String,
    pub uname: String,
}

impl EntityName {
    fn new(name: String) -> Self {
        Self {
            uname: name.to_uppercase(),
            name
        }
    }
}


#[derive(Clone)]
pub struct Entity {
    // An entity's {ID, Generation} combination is its unique identifier. IDs begin at 0.
    // When an entity is deleted, its Kind is set to None. When the ID is re-used, generation is
    // incremented by one.
    pub id: usize,
    pub generation: usize,

    // All entities must have a name... an upper-case version of the name is stored for indexing.
    // Only Garbage has no name.
    pub name: Option<EntityName>,

    // Entities which belong to the same namespace must all have unique unames.
    // If namespace is None, no uniqueness enforcement is performed.
    pub namespace: Option<String>,

    // Entities may have Option aliases. These are also enforced in the namespace if one is set.
    pub aliases: Vec<EntityName>,

    // Entities either have a KIND (which is denoted by String) or they do not. if they do NOT have a Kind,
    // they are GARBAGE/UNUSED.
    pub kind: Option<String>,

    pub creation_timestamp: SystemTime,
    pub modified_timestamp: SystemTime,

    // Entities either have a Location or they do not. Entities without a location are just
    // floating in nullspace.
    pub location: Option<EntityLocation>,

    // All Entities have an owner. Without one, it 'owns itself'.
    pub owner: Option<usize>,

    // All Entities can have a Home which they will be sent to as a primary fallback should their
    // location become unexpectedly invalid.
    pub home: Option<EntityLocation>,

    // Exits should have a Destination.
    pub destination: Option<EntityLocation>,

    // admin_level is a low-level register of the permissions of this object. Exactly what it means
    // depends on game implementation. Generally, higher level means more admin power.
    pub admin_level: usize,

    // The below members are used for indexing and are not saved directly to database.
    pub contents: HashSet<usize>,
    pub belongings: HashSet<usize>,
    pub entrances: HashSet<usize>
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            id: 0,
            generation: 1,
            name: None,
            namespace: None,
            aliases: vec![],
            kind: None,
            creation_timestamp: SystemTime::now(),
            modified_timestamp: SystemTime::now(),
            location: None,
            owner: None,
            home: None,
            destination: None,
            admin_level: 0,
            contents: Default::default(),
            belongings: Default::default(),
            entrances: Default::default()
        }
    }
}


impl Entity {
    fn next_gen(&self) -> usize {
        self.generation + 1
    }
    
    // Turns this object into garbage. Note: should be called by the EntityManager, not directly.
    fn trash(&mut self) {
        self.name = None;
        self.namespace = None;
        self.aliases.clear();
        self.kind = None;
        self.location = None;
        self.owner = None;
        self.home = None;
        self.destination = None;
        self.admin_level = 0;
        self.contents.clear();
        self.belongings.clear();
        self.entrances.clear();
    }
    
}

pub struct NameSpace {
    // List of every entity that is using this namespace.
    pub entities: HashSet<usize>,

    // HashMap of all names currently in use for this namespace, in upper-case format.
    pub names: HashMap<String, usize>
}

pub struct Kind {
    pub name: String,
    pub entities: HashSet<usize>,
    // Probably should put in something like a Python path to register here? If I'm using python?
}

pub struct EntityManager {
    // Vector of all Entities in system. Note that this is always accessed by index and some elements
    // are Garbage entities ready to be re-created.
    pub entities: Vec<Entity>,

    // Namespaces are used to index uniquely-named entities such as 'PLAYERS' or 'GUILDS'.
    pub namespaces: HashMap<String, NameSpace>,

    pub kinds: HashMap<String, Kind>,

    pub garbage: HashSet<usize>
}

impl EntityManager {

    // Will return the ID it's given if it is aavailable for creation. If it is not available, will
    // return something else that's usable.
    fn available_id(&self, id_choice: Option<usize>) -> usize {
        match id_choice {
            Some(id_c) => {
                if self.entities.len() >= id_c {
                    // The entered number is available...?
                    if self.garbage.contains(&id_c) {
                        // Number is available for re-use!
                        id_c
                    }
                    else {
                        // Number is not available. generate new one.
                        self.entities.len() + 1
                    }
                }
                else {
                    // The entered number is too high. Generate the next highest available.
                    self.entities.len() + 1
                }
            }
            None => {
                // No number was entered. Generate a new one.
                self.entities.len() + 1
            }
        }
    }
    
    // Method to do the actual low-level construction of an Entity
    // Returns the ID of the created entity.
    fn create_entity(&mut self, name: String, namespace: Option<String>, kind: String, location: Option<EntityLocation>,
    destination: Option<EntityLocation>, home: Option<EntityLocation>, owner: Option<usize>, id: usize) -> usize {

        // fn available_id must already have been used to validate/generate an ID.

        let ent_name = EntityName::new(name);

        let mut new_ent = Entity::default();
        new_ent.id = id;
        new_ent.name = Some(ent_name.clone());
        new_ent.namespace = namespace.clone();
        new_ent.kind = Some(kind.clone());
        new_ent.location = location.clone();
        new_ent.owner = owner.clone();
        new_ent.home = home.clone();
        new_ent.destination = destination.clone();

        if self.garbage.contains(&id) {
            let mut old_ent = self.entities.get_mut(id).unwrap();
            new_ent.generation = old_ent.next_gen();
            self.entities[id] = new_ent;
        }

        else {
            let available = self.entities.capacity() - self.entities.len();
            if available == 0 {
                self.entities.reserve(100);
            }
            self.entities.push(new_ent);
        }
        
        //let mut ent = self.entities.get_mut(id).unwrap();

        // Index namespace.
        if let Some(nspace) = namespace {
            if let Some(nspa) = self.namespaces.get_mut(&nspace) {
                nspa.entities.insert(id);
                nspa.names.insert(ent_name.uname.clone(), id);
            }
        }

        if let Some(own) = owner {
            if let Some(own_ent) = self.entities.get_mut(own) {
                own_ent.belongings.insert(id);
            }
        }

        if let Some(dest) = destination {
            if let Some(dest) = self.entities.get_mut(dest.location) {
                dest.entrances.insert(id);
            }
        }

        if let Some(loc) = location {
            if let Some(holder) = self.entities.get_mut(loc.location) {
                holder.contents.insert(id);
            }
        }

        if let Some(kind) = self.kinds.get_mut(&kind) {
            kind.entities.insert(id);
        }
        id
    }
    
    // Performs actual deletion of an Entity. This clears its properties, wipes cache, and readies
    // the ID for re-use.
    fn delete_entity(&mut self, id: usize) {

        let mut ent = self.entities.get_mut(id).unwrap().clone();
        let loc = ent.location.clone();
        let own = ent.owner.clone();
        let dest = ent.destination.clone();
        let name = ent.name.clone();


        if let Some(loc) = loc {
            if let Some(holder) = self.entities.get_mut(loc.location) {
                holder.contents.remove(&id);
            }
        }

        if let Some(own) = own {
            if let Some(owner) = self.entities.get_mut(own) {
                owner.belongings.remove(&id);
            }
        }

        if let Some(dest) = dest {
            if let Some(dest) = self.entities.get_mut(dest.location) {
                dest.entrances.remove(&id);
            }
        }

        if let Some(nm) = name {
            if let Some(nspace) = &ent.namespace {
                if let Some(nsp) = self.namespaces.get_mut(nspace) {
                    nsp.entities.remove(&id);
                    nsp.names.remove(&nm.uname);
                }
            }
        }

        if let Some(kind) = &ent.kind {
            if let Some(knd) = self.kinds.get_mut(kind) {
                knd.entities.remove(&id);
            }
        }

        ent.trash();
        self.garbage.insert(id);
    }
}