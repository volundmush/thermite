use std::collections::{HashSet, HashMap};
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;
use super::typedefs::Dbref;

#[derive(Debug)]
pub struct Restriction {
    pub name: &'static str,
    pub command: bool,
    pub function: bool,
}

#[derive(Debug)]
pub struct RestrictionManager {
    pub restrictions: HashMap<&'static str, Rc<Restriction>>,
    pub command_restrictions: HashMap<&'static str, Rc<Restriction>>,
    pub function_restrictions: HashMap<&'static str, Rc<Restriction>>
}

impl RestrictionManager {
    fn add_restriction(&mut self, restrict: Restriction) {
        let r = Rc::new(restrict);
        if r.command {
            self.command_restrictions.insert(r.name, r.clone());
        };
        if r.function {
            self.function_restrictions.insert(r.name, r.clone());
        };
        self.restrictions.insert(r.name, r);
    }
}

impl Default for RestrictionManager {
    fn default() -> Self {
        let mut manager = Self {
            restrictions: Default::default(),
            command_restrictions: Default::default(),
            function_restrictions: Default::default()
        };

        manager.add_restriction(Restriction {
            name: "god",
            function: true,
            command: true
        });

        manager.add_restriction(Restriction {
            name: "wizard",
            function: true,
            command: true
        });

        manager.add_restriction(Restriction {
            name: "admin",
            function: true,
            command: true
        });

        manager.add_restriction(Restriction {
            name: "nogagged",
            function: true,
            command: true
        });

        manager.add_restriction(Restriction {
            name: "nofixed",
            function: true,
            command: true
        });

        manager.add_restriction(Restriction {
            name: "noguest",
            function: true,
            command: true
        });

        manager.add_restriction(Restriction {
            name: "nobody",
            function: true,
            command: true
        });

        manager.add_restriction(Restriction {
            name: "logname",
            function: true,
            command: true
        });

        manager.add_restriction(Restriction {
            name: "logargs",
            function: true,
            command: true
        });

        manager.add_restriction(Restriction {
            name: "noparse",
            function: true,
            command: false
        });

        manager.add_restriction(Restriction {
            name: "localize",
            function: true,
            command: false
        });

        manager.add_restriction(Restriction {
            name: "userfn",
            function: true,
            command: false
        });

        manager.add_restriction(Restriction {
            name: "nosidefx",
            function: true,
            command: false
        });

        manager.add_restriction(Restriction {
            name: "deprecated",
            function: true,
            command: false
        });

        manager.add_restriction(Restriction {
            name: "noplayer",
            function: false,
            command: true
        });

        manager
    }
}