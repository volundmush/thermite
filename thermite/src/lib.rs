#[macro_use]
extern crate diesel;
use diesel::prelude::*;

pub mod config;
pub mod db;
pub mod schema;
pub mod models;
mod conn;