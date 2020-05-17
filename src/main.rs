#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate tokio;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use tokio::prelude::*;
use std::env;
use thermite::networking;
use thermite::engine;
use thermite::engine::ThermiteEngine;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));
    println!("Hello, world!");

    let mut myengine = ThermiteEngine {};
    let mut connection_manager = networking::ConnectionManger { connections: vec![],
        handlers: Default::default(), engine: &myengine};
    println!("It's amazing we got this far.");
}
