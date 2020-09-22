#[macro_use]
extern crate diesel;
use diesel::prelude::*;

//pub mod evstring;
pub mod config;
pub mod db;
pub mod schema;
pub mod models;
pub mod lobby;
pub mod commands;