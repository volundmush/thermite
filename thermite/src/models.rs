use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};

#[derive(Queryable)]
pub struct User {
    pub id: isize,
    pub username: String,
    pub date_joined: NaiveDateTime,
    pub password_hash: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub email_verified_on: Option<NaiveDateTime>,
    pub active: bool,
    pub is_superuser: bool,
    pub is_admin: bool,
    pub timezone: String,
    pub banned_until: Option<NaiveDateTime>,
    pub banned_by: Option<isize>,
    pub ban_reason: Option<String>
}

#[derive(Queryable)]
pub struct UserSession {
    pub id: isize,
    pub user_id: isize,
    pub created: NaiveDateTime,
    pub valid_until: NaiveDateTime,
    pub session_key: String
}

#[derive(Queryable)]
pub struct UserStorage {
    pub id: isize,
    pub user_id: isize,
    pub category: String,
    pub storage_name: String,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub json_data: serde_json::Value
}

#[derive(Queryable)]
pub struct Game {
    pub id: isize,
    pub user_id: isize,
    pub abbr: String,
    pub display_name: String,
    pub created: NaiveDateTime,
    pub active: bool,
    pub is_public: bool,
    pub game_key: String,
    pub banned_until: Option<NaiveDateTime>,
    pub banned_by: Option<isize>,
    pub ban_reason: Option<String>
}

#[derive(Queryable)]
pub struct GameMember {
    pub id: isize,
    pub game_id: isize,
    pub user_id: isize,
    pub joined: NaiveDateTime,
    pub member_key: String,
    pub active: bool,
    pub is_superuser: bool,
    pub is_admin: bool,
    pub banned_until: Option<NaiveDateTime>,
    pub banned_by: Option<isize>,
    pub ban_reason: Option<String>
}

#[derive(Queryable)]
pub struct MemberStorage {
    pub id: isize,
    pub member_id: isize,
    pub category: String,
    pub storage_name: String,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub json_data: Option<serde_json::Value>
}

#[derive(Queryable)]
pub struct Board {
    pub id: isize,
    pub board_id: isize,
    pub game_id: Option<isize>,
    pub name: String,
    pub display_name: String,
    pub mandatory: bool,
    pub restricted: u8,
    pub next_id: isize
}

#[derive(Queryable)]
pub struct Post {
    pub id: isize,
    pub board_id: isize,
    pub post_num: isize,
    pub user_id: isize,
    pub subject: String,
    pub body: String,
    pub date_created: NaiveDateTime,
    pub date_modified: NaiveDateTime
}

#[derive(Queryable)]
pub struct PostRead {
    pub id: isize,
    pub post_id: isize,
    pub user_id: isize,
    pub date_checked: NaiveDateTime
}

#[derive(Queryable)]
pub struct Channel {
    pub id: isize,
    pub game_id: Option<isize>,
    pub name: String,
    pub display_name: String,
    pub restricted: u8
}

#[derive(Queryable)]
pub struct ChannelSub {
    pub id: isize,
    pub channel_id: isize,
    pub user_id: isize,
    pub command: String,
    pub status: u8
}