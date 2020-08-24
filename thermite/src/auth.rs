use diesel::{
    prelude::*,
    r2d2::Pool
};
use crate::schema;
use tokio_diesel::*;
use chrono::NaiveDateTime;
use tokio::{
    sync::mpsc::{channel, Sender, Receiver}
};

// Request creation of a new user with this struct.
pub struct ReqUserCreate {
    pub username: String,
    pub password: String
}

// Request adding an email to a new user with this struct.
pub struct ReqAddEmail {
    pub user_id: i32,
    pub email: String,
    pub make_primary: bool
}

// Request removing an email with this struct.
pub struct ReqRemEmail {
    pub user_id: i32,
    pub user_email_id: i32
}


// Request to change primary email.
pub struct ReqPrimEmail {
    pub user_id: i32,
    pub user_email_id: i32
}

pub struct ReqBanUser {
    pub user_id: i32,
    pub banned_by: i32,
    pub ban_reason: String,
    pub ban_until: NaiveDateTime
}

pub struct ReqDisableUser {
    pub user_id: i32
}

pub struct ReqEnableUser {
    pub user_id: i32
}

pub struct ReqSetPassword {
    pub user_id: i32,
    pub new_password: String
}

pub struct ReqSetStorage {
    pub user_id: i32,
    pub category: String,
    pub storage_name: String,
    pub json_data: serde_json::Value
}

pub struct ReqDelStorage {
    pub user_id: i32,
    pub category: String,
    pub storage_name: String
}

pub struct ReqWipeStorage {
    pub user_id: i32,
    pub category: String
}

pub enum Msg2UserManager {
    CreateUser(ReqUserCreate),
    AddEmail(ReqAddEmail),
    RemEmail(ReqRemEmail),
    PrimaryEmail(ReqPrimEmail),
    EnableUser(ReqEnableUser),
    DisableUser(ReqDisableUser),
    SetPassword(ReqSetPassword),
    SetStorage(ReqSetStorage),
    DelStorage(ReqDelStorage),
    WipeStorage(ReqWipeStorage),
    Kill
}

pub struct UserManager {
    rx_usermanager: Receiver<Msg2UserManager>
}

impl UserManager {
    pub async fn run(&mut self) {
        loop {
            if let Some(msg) = self.rx_usermanager.recv().await {
                match msg {
                    Msg2UserManager::CreateUser(req) => {

                    },
                    Msg2UserManager::AddEmail(req) => {

                    },
                    Msg2UserManager::RemEmail(req) => {

                    },
                    Msg2UserManager::PrimaryEmail(req) => {

                    },
                    Msg2UserManager::EnableUser(req) => {

                    },
                    Msg2UserManager::DisableUser(req) => {

                    },
                    Msg2UserManager::SetPassword(req) => {

                    },
                    Msg2UserManager::SetStorage(req) => {

                    },
                    Msg2UserManager::DelStorage(req) => {

                    },
                    Msg2UserManager::WipeStorage(req) => {

                    },
                    Msg2UserManager::Kill => {
                        break;
                    }
                }
            }
        }
    }
}