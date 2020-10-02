use legion::*;
use crate::{
    components::props::*
};
use legion::storage::Component;


pub fn find_name<T: Component>(world: &World, name: &str) -> Option<Entity> {
    let mut q = <(&T, &PropName)>::query();

    for (_, namecom) in q.iter(world) {
        if namecom.name == name {
            return Some(namecom.entity)
        }
    }
    None
}

pub fn get_or_create<T: Component + Default>(world: &mut World, name: &str) -> Entity {
    let cleaned = name.trim().to_uppercase();
    return if let Some(ent) = find_name::<T>(world, &cleaned) {
        ent
    } else {
        let ent = world.push((T::default(),));
        if let Some(mut entry) = world.entry(ent) {
            entry.add_component(PropName::new(ent, &cleaned));
        }
        ent
    }
}