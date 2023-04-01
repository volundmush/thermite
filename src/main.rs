use tokio::{
    net::TcpListener
};

use std::{
    error::Error,
    net::{SocketAddr}
};

use thermite::{
    config::{Config},
    net::{ListenManager},
    telnet::{
        factory::TelnetProtocolFactory
    },
    link::{
        factory::LinkProtocolFactory
    },
    portal::{Portal}
};


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

    let mut telnet_factory = TelnetProtocolFactory::new("telnet", tx_portal.clone());
    listen.register_factory(telnet_factory.link());
    let _ = tokio::spawn(async move {telnet_factory.run().await});

    let mut link_factory = LinkProtocolFactory::new("link", tx_portal.clone());
    listen.register_factory(link_factory.link());
    let _ = tokio::spawn(async move {link_factory.run().await});


    for (k, v) in conf.listeners.iter() {

        let addr = conf.interfaces.get(&v.interface).expect("Telnet Server attempting to use non-existent interface!");
        let sock = SocketAddr::new(addr.clone(), v.port);
        let listener = TcpListener::bind(sock).await.expect("Could not bind server port... is it in use?");

        if let Some(tls_key) = &v.tls {
            // Will worry about TLS later...
        } else {
            listen.listen(k, listener, None, &v.protocol.clone());
        }
    }

    let _ = tokio::spawn(async move {listen.run().await});
    let portal_task = tokio::spawn(async move {portal.run().await});

    let _ = portal_task.await;

    Ok(())
}