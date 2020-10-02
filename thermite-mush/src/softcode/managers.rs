use std::{
    collections::{HashMap, HashSet},
    cmp::max
};

use crate::{
    softcode::typedefs::{DbRef, DbError}
};

use legion::Entity;

#[derive(Default)]
pub struct DbRefManager {
    pub greatest: usize,
    pub available: Vec<usize>,
    pub dbrefs: HashMap<DbRef, Entity>,
}

impl DbRefManager {
    pub fn init(&mut self) {
        for (i, o) in self.dbrefs.iter().filter(|(i, o)| i.is_num()) {
            self.greatest = max(self.greatest, i.to_num());
        }
    }

    pub fn scan_recycle(&mut self) {
        for i in 0..self.greatest {
            let db = DbRef::from(i);
            if !self.dbrefs.contains_key(&db) {
                self.available.push(i);
                if self.available.len() > 50 {
                    break
                }
            }
        }
    }

    pub fn first_available(&mut self) -> DbRef {
        if let Some(avail) = self.available.pop() {
            DbRef::from(avail)
        } else {
            self.scan_recycle();
            if let Some(avail) = self.available.pop() {
                DbRef::from(avail)
            } else {
                DbRef::from(self.greatest + 1)
            }
        }
    }

    pub fn create(&mut self, choice: DbRef) -> Result<DbRef, DbError> {
        // given a Dbref, return a valid DbRef to use or Error. If given a SPECIFIC DbRef, it is
        // available if it is not taken and is <= Greatest. Given a DbRef::None, pick the first available.

        if self.dbrefs.contains_key(&choice) {
            return Err(DbError::new("dbref already in use"))
        }

        match &choice {
            DbRef::None => Ok(self.first_available()),
            DbRef::Num(n) => {
                if *n > self.greatest {
                    return Err(DbError::new("dbref higher than highest used"))
                } else {
                    Ok(choice)
                }
            },
            DbRef::Name(usize) => Ok(choice)
        }
    }

    pub fn delete(&mut self, choice: &DbRef) -> Option<Entity> {
        let result = self.dbrefs.remove(choice);
        if result.is_some() {
            if choice.is_num() {
                if self.available.len() <= 49 {
                    self.available.push(choice.to_num());
                }
            }
        }
        result
    }
}