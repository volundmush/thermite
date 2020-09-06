use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};

#[derive(Queryable)]
pub struct Game {
    pub id: i32,
    pub user_id: i32,
    pub abbr: String,
    pub display_name: String,
    pub created: NaiveDateTime,
    pub active: bool,
    pub is_public: bool,
    pub game_key: String,
    pub banned_until: Option<NaiveDateTime>,
    pub banned_by: Option<i32>,
    pub ban_reason: Option<String>
}

#[derive(Queryable)]
pub struct GameMember {
    pub id: i32,
    pub game_id: i32,
    pub user_id: i32,
    pub joined: NaiveDateTime,
    pub member_key: String,
    pub active: bool,
    pub is_superuser: bool,
    pub is_admin: bool,
    pub banned_until: Option<NaiveDateTime>,
    pub banned_by: Option<i32>,
    pub ban_reason: Option<String>
}

#[derive(Queryable)]
pub struct MemberStorage {
    pub id: i32,
    pub member_id: i32,
    pub category: String,
    pub storage_name: String,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub json_data: serde_json::Value
}

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub joined: NaiveDateTime,
    pub password_hash: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub email_verified_on: Option<NaiveDateTime>,
    pub active: bool,
    pub is_superuser: bool,
    pub is_admin: bool,
    pub lang_tag: String,
    pub timezone: String,
    pub banned_until: Option<NaiveDateTime>,
    pub banned_by: Option<i32>,
    pub ban_reason: Option<String>
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

#[derive(Queryable)]
pub struct Board {
    pub id: i32,
    pub board_id: i32,
    pub game_id: Option<i32>,
    pub name: String,
    pub display_name: String,
    pub mandatory: bool,
    pub restricted: u8,
    pub next_id: i32
}

#[derive(Queryable)]
pub struct Post {
    pub id: i32,
    pub board_id: i32,
    pub post_num: i32,
    pub subject: String,
    pub text: String,
    pub date_created: NaiveDateTime,
    pub date_modified: NaiveDateTime
}

#[derive(Queryable)]
pub struct PostRead {
    pub id: i32,
    pub post_id: i32,
    pub date_checked: NaiveDateTime
}

#[derive(Queryable)]
pub struct Channel {
    pub id: i32,
    pub game_id: Option<i32>,
    pub name: String,
    pub display_name: String,
    pub restricted: u8
}

#[derive(Queryable)]
pub struct ChannelSub {
    pub id: i32,
    pub channel_id: i32,
    pub user_id: i32,
    pub command: String,
    pub status: u8
}