use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};

#[derive(Queryable)]
pub struct Game {
    pub id: i32,
    pub user_id: i32,
    pub gamename: String,
    pub display_name: String,
    pub created: NaiveDateTime,
    pub is_public: bool
}

#[derive(Queryable)]
pub struct GameApi {
    pub id: i32,
    pub game_id: i32,
    pub game_key: String
}

#[derive(Queryable)]
pub struct GameBan {
    pub id: i32,
    pub game_id: i32,
    pub user_id: i32,
    pub banned_on: NaiveDateTime,
    pub banned_until: NaiveDateTime,
    pub banned_by: Option<i32>,
    pub ban_reason: String
}

#[derive(Queryable)]
pub struct GameMember {
    pub id: i32,
    pub game_id: i32,
    pub user_id: i32,
    pub joined: NaiveDateTime,
    pub member_key: String
}

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub joined: NaiveDateTime,
    pub password_hash: Option<String>,
    pub active: bool,
    pub is_supervisor: bool
}

#[derive(Queryable)]
pub struct UserBan {
    pub id: i32,
    pub user_id: i32,
    pub banned_on: NaiveDateTime,
    pub banned_until: NaiveDateTime,
    pub banned_by: Option<i32>,
    pub ban_reason: String
}

#[derive(Queryable)]
pub struct Email {
    pub id: i32,
    pub address: String,
    pub added: NaiveDateTime,
    pub verified: bool,
    pub verified_at: Option<NaiveDateTime>
}

#[derive(Queryable)]
pub struct UserEmail {
    pub id: i32,
    pub user_id: i32,
    pub email_id: i32,
    pub added: NaiveDateTime
}

#[derive(Queryable)]
pub struct UserPassword {
    pub id: i32,
    pub user_id: i32,
    pub password_hash: String,
    pub added: NaiveDateTime
}

#[derive(Queryable)]
pub struct UserProfile {
    pub id: i32,
    pub user_id: i32,
    pub display_name: Option<String>,
    pub email: Option<i32>,
    pub lang_tag: String,
    pub timezone: String
}

#[derive(Queryable)]
pub struct UserSession {
    pub id: i32,
    pub user_id: i32,
    pub created: NaiveDateTime,
    pub valid_until: NaiveDateTime,
    pub session_key: String
}

#[derive(Queryable)]
pub struct UserStorage {
    pub id: i32,
    pub user_id: i32,
    pub category: String,
    pub storage_name: String,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub json_data: serde_json::Value
}