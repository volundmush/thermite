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
    net::{IpAddr, SocketAddr}
};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ServerConfig,
        PrivateKey,
        Certificate,
        NoClientAuth,
        internal::pemfile::{
            certs,
            rsa_private_keys
        }
    },
};

use thermite::config::{Config, ServerConfig as ThermiteServer};
use std::str::FromStr;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conf = Config::from_file(String::from("config.toml"))?;

    let mut tls: HashMap<String, TlsAcceptor> = HashMap::with_capacity(conf.tls.len());
    for (k, v) in conf.tls.iter() {
        // I'll worry about this later..
    }

    let mut interfaces: HashMap<String, IpAddr> = HashMap::with_capacity(conf.interfaces.len());
    for (k, v) in conf.interfaces.iter() {
        let addr = IpAddr::from_str(v).expect("Could not validate IP address!");
        interfaces.insert(k.clone(), addr);
    }
    // Setup PostGres via Diesel and Tokio-Diesel
    let database_url = conf.database.get("postgres").expect("No database configured for 'postgres'!");
    let db_manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_pool = Pool::builder().build(db_manager).expect("Could not start Database connection pool.");

    let mut telnet_server = TelnetServer::new();
    let tx_telnet = telnet_server.tx_server.clone();

    for (k, v) in conf.telnet.iter() {
        let addr = interfaces.get(&v.interface).expect("Telnet Server attempting to use non-existent interface!");
        let sock = SocketAddr::new(addr.clone(), v.port);
        let listener = TcpListener::bind(sock).await.expect("Could not bind Telnet Server port... is it in use?");
        if let Some(tls_key) = &v.tls {
            // Will worry about TLS later...
        } else {
            telnet_server.listen(String::from(k), listener, None);
        }
    }
    //let telnet_task = tokio::spawn(async move {telnet_server.run();});
    //telnet_task.await;
    telnet_server.run().await;
    Ok(())
}