use diesel::{
    prelude::*,
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool}
};

use tokio::{
    net::TcpListener,
    sync::mpsc::{channel, Receiver, Sender}
};

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

use thermite_lib::conn::{Portal, ProtocolType};
use thermite_telnet::{codes as tc};

use thermite::{
    config::{Config, ServerConfig as ThermiteServer},
    db::DbManager,
    telnet::TelnetOption,
    session::SessionManager
};
use std::str::FromStr;


fn teloptions() -> HashMap<u8, TelnetOption> {
    let mut map: HashMap<u8, TelnetOption> = Default::default();

    map.insert(tc::SGA, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    map.insert(tc::NAWS, TelnetOption {allow_local: false, allow_remote: true, start_remote: true, start_local: false});
    map.insert(tc::TTYPE, TelnetOption {allow_local: false, allow_remote: true, start_remote: true, start_local: false});
    //map.insert(tc::MXP, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    map.insert(tc::MSSP, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    map.insert(tc::MCCP2, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    //map.insert(tc::MCCP3, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    map.insert(tc::GMCP, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    map.insert(tc::MSDP, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    map.insert(tc::LINEMODE, TelnetOption {allow_local: false, allow_remote: true, start_remote: true, start_local: false});
    map.insert(tc::TELOPT_EOR, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    map
}


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

    let mut db = DbManager::new(db_pool);
    let tx_dbmanager = db.tx_dbmanager.clone();

    let db_task = tokio::spawn(async move {db.run().await});

    let (tx_sessmanager, rx_sessmanager) = channel(50);
    let (tx_portal, rx_portal) = channel(50);

    let mut sess_manager = SessionManager::new(tx_sessmanager.clone(), rx_sessmanager, tx_portal.clone());
    let sess_task = tokio::spawn(async move {sess_manager.run().await});

    let mut portal = Portal::new(tx_portal.clone(), rx_portal, tx_sessmanager.clone());

    for (k, v) in conf.listeners.iter() {

        let protocol = match v.protocol.to_lowercase().as_ref() {
            "telnet" => ProtocolType::Telnet,
            "websocket" => ProtocolType::WebSocket,
            _ => panic!("Unsupported Protocol type {}", v.protocol)
        };

        let addr = interfaces.get(&v.interface).expect("Telnet Server attempting to use non-existent interface!");
        let sock = SocketAddr::new(addr.clone(), v.port);
        let listener = TcpListener::bind(sock).await.expect("Could not bind Telnet Server port... is it in use?");

        if let Some(tls_key) = &v.tls {
            // Will worry about TLS later...
        } else {
            portal.listen(String::from(k), listener, None, protocol);
        }
    }
    let portal_task = tokio::spawn(async move {portal.run().await});
    portal_task.await;

    Ok(())
}