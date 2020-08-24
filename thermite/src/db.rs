use diesel::{
    prelude::*,
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool}
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

pub struct ReqUnbanUser {
    pub user_id: i32
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

pub enum Msg2DbManager {
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
    BanUser(ReqBanUser),
    UnbanUser(ReqUnbanUser),
    Kill
}

pub struct DbManager{
    rx_dbmanager: Receiver<Msg2DbManager>,
    pub tx_dbmanager: Sender<Msg2DbManager>,
    pool: Pool<ConnectionManager<PgConnection>>
}

impl DbManager {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        let (tx_dbmanager, rx_dbmanager) = channel(50);
        Self {
            tx_dbmanager,
            rx_dbmanager,
            pool
        }
    }

    pub async fn run(&mut self) -> () {
        loop {
            if let Some(msg) = self.rx_dbmanager.recv().await {
                match msg {
                    Msg2DbManager::CreateUser(req) => {

                    },
                    Msg2DbManager::AddEmail(req) => {

                    },
                    Msg2DbManager::RemEmail(req) => {

                    },
                    Msg2DbManager::PrimaryEmail(req) => {

                    },
                    Msg2DbManager::EnableUser(req) => {

                    },
                    Msg2DbManager::DisableUser(req) => {

                    },
                    Msg2DbManager::SetPassword(req) => {

                    },
                    Msg2DbManager::SetStorage(req) => {

                    },
                    Msg2DbManager::DelStorage(req) => {

                    },
                    Msg2DbManager::WipeStorage(req) => {

                    },
                    Msg2DbManager::BanUser(req) => {

                    },
                    Msg2DbManager::UnbanUser(req) => {

                    }
                    Msg2DbManager::Kill => {
                        break;
                    }
                }
            }
        }
    }
}