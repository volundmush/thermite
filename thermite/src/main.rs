use diesel::{
    prelude::*,
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool}
};

use thermite_telnet::telnet::{TelnetServer};
use tokio::net::TcpListener;
use std::{
    env,
    error::Error,
    collections::HashMap,
    net::SocketAddr
};
use tokio_rustls::TlsAcceptor;

use thermite::config::Config;




#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conf = Config::from_file(String::from("config.toml"))?;

    let tls: HashMap<String, TlsAcceptor> = HashMap::default();
    for (k, v) in conf.tls.iter() {

    }

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder().build(manager)?;

    let listener = TcpListener::bind("0.0.0.0:7999").await.unwrap();
    let mut srv = TelnetServer::new();

    srv.listen(String::from("telnet"), listener, None);

    srv.run().await;
    Ok(())
}