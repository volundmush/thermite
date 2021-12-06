use tokio::{
    net::TcpListener
};

use std::{
    error::Error,
    collections::HashMap,
    net::{SocketAddr}
};

use thermite::{
    config::{Config, ServerConfig as ThermiteServer},
    net::{ListenManager},
    telnet::{
        codes as tc,
        protocol::TelnetOption,
        factory::TelnetProtocolFactory
    },
    link::{
        factory::LinkProtocolFactory
    },
    portal::{Portal, Msg2Portal}
};


fn teloptions() -> HashMap<u8, TelnetOption> {
    let mut map: HashMap<u8, TelnetOption> = Default::default();

    map.insert(tc::SGA, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    map.insert(tc::NAWS, TelnetOption {allow_local: false, allow_remote: true, start_remote: true, start_local: false});
    map.insert(tc::TTYPE, TelnetOption {allow_local: false, allow_remote: true, start_remote: true, start_local: false});
    //map.insert(tc::MXP, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    map.insert(tc::MSSP, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
    //map.insert(tc::MCCP2, TelnetOption {allow_local: true, allow_remote: true, start_remote: false, start_local: true});
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

    //let mut tls: HashMap<String, TlsAcceptor> = HashMap::with_capacity(conf.tls.len());
    for (k, v) in conf.tls.iter() {
        // I'll worry about this later..
    }

    let mut portal = Portal::new();
    let tx_portal = portal.tx_portal.clone();
    //let prot_task = tokio::spawn(async move {prot_manager.run().await});

    let mut listen = ListenManager::new();

    let mut telnet_factory = TelnetProtocolFactory::new("telnet".parse().unwrap(), teloptions(), tx_portal.clone());
    listen.register_factory(telnet_factory.link());
    let _tel_task = tokio::spawn(async move {telnet_factory.run().await});

    let mut link_factory = LinkProtocolFactory::new("link".parse().unwrap(), tx_portal.clone());
    listen.register_factory(link_factory.link());
    let _link_task = tokio::spawn(async move {link_factory.run().await});


    for (k, v) in conf.listeners.iter() {

        let addr = conf.interfaces.get(&v.interface).expect("Telnet Server attempting to use non-existent interface!");
        let sock = SocketAddr::new(addr.clone(), v.port);
        let listener = TcpListener::bind(sock).await.expect("Could not bind server port... is it in use?");

        if let Some(tls_key) = &v.tls {
            // Will worry about TLS later...
        } else {
            listen.listen(String::from(k), listener, None, &v.protocol.clone());
        }
    }

    let listen_task = tokio::spawn(async move {listen.run().await});
    let _ = portal.start_timer().await;
    let portal_task = tokio::spawn(async move {portal.run().await});

    portal_task.await;

    Ok(())
}