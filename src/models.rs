use diesel::pg::types::sql_types::Uuid;

#[derive(Queryable)]
pub struct PluginName {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable)]
pub struct EntityType {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable)]
pub struct Entity {
    pub id: Uuid
}