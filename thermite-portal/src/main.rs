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

use thermite_net::{Portal, Msg2Portal};
use thermite_telnet::{
    codes as tc,
    protocol::TelnetOption,
};

use thermite::{
    config::{Config, ServerConfig as ThermiteServer},
    db::DbManager,
    lobby::Lobby
};
use thermite_protocol::{
    telnet::{TelnetOption,TelnetProtocolFactory}
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
    // #TODO: Fix MCCP3. It doesn't work... why?
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


    // Setup Sqlite3 via Diesel and Tokio-Diesel
    let database_url = conf.database.get("sqlite3").expect("No database configured for 'sqlite3'!");
    let mut db = DbManager::new(database_url.clone()).expect("Could not start Database connection pool.");
    let tx_dbmanager = db.tx_dbmanager.clone();

    let db_task = tokio::spawn(async move {db.run().await});

    let mut prot_manager = ProtocolManager::new(tx_dbmanager.clone());
    let tx_manager = prot_manager.tx_manager.clone();
    let prot_task = tokio::spawn(async move {prot_manager.run().await});

    let mut portal = Portal::new(None);

    let mut telnet_factory = TelnetProtocolFactory::new("telnet".parse().unwrap(), teloptions(), tx_manager.clone());
    portal.register_factory(telnet_factory.link());

    let tel_task = tokio::spawn(async move {telnet_factory.run().await});

    for (k, v) in conf.listeners.iter() {

        let addr = conf.interfaces.get(&v.interface).expect("Telnet Server attempting to use non-existent interface!");
        let sock = SocketAddr::new(addr.clone(), v.port);
        let listener = TcpListener::bind(sock).await.expect("Could not bind server port... is it in use?");

        if let Some(tls_key) = &v.tls {
            // Will worry about TLS later...
        } else {
            portal.listen(String::from(k), listener, None, &v.protocol.clone());
        }
    }
    let portal_task = tokio::spawn(async move {portal.run().await});
    portal_task.await;

    Ok(())
}