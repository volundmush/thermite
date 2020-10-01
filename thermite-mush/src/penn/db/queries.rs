use legion::*;
use super::{
    components::*
};


pub fn find_property_name<T: Component>(name: &str, world: &World) -> Option<Entity> {
    let q = <(&T, &NameComponent)>::query();

    for (_, namecom) in q.iter(&world) {
        if namecom.name == name {
            return Some(namecom.entity)
        }
    }
    None
}

pub fn get_or_create_property<T: Component + Default>(name: &str, world: &mut World) -> Entity {
    if let Some(ent) = find_property_name::<T>(name, &world) {
        return ent
    } else {
        let ent = world.push(T::default());
        if let Some(mut entry) = world.entries(ent) {
            entry.add_component(NameComponent::new(name, ent));
        }
        return ent
    }
}