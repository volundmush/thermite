use legion::Entity;

#[derive(Default, Debug)]
pub struct ObjTypeMarker;
#[derive(Default, Debug)]
pub struct RestrictMarker;
#[derive(Default, Debug)]
pub struct AttrFlagMarker;
#[derive(Default, Debug)]
pub struct FlagPermMarker;
#[derive(Default, Debug)]
pub struct LockFlagMarker;
#[derive(Default, Debug)]
pub struct CmdFlagMarker;
#[derive(Default, Debug)]
pub struct FlagMarker;
#[derive(Default, Debug)]
pub struct PowerMarker;
#[derive(Default, Debug)]
pub struct AttributeMarker;


#[derive(Debug)]
pub struct PropName {
    pub entity: Entity,
    pub name: String,
}

impl PropName {
    pub fn new(ent: Entity, name: &str) -> Self {
        Self {
            name: name.trim().to_uppercase(),
            entity: ent
        }
    }
}