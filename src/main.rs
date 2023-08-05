use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
};

use std::sync::Arc;
use clap::{Parser};
use futures::future::join_all;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use rustls_pemfile::{certs, pkcs8_private_keys};


use tracing::{error, info, Level};
use tracing_subscriber;

use thermite::{
    networking::{
        link::LinkAcceptor,
        telnet::TelnetAcceptor,
        web::run_warp
    },
    IS_TLS_ENABLED,
    TX_PORTAL
};

use thermite::portal::Portal;



#[derive(Parser, Debug)]
#[clap(version = "0.1", author = "Andrew Bastien <volundmush@gmail.com>", about = "A networking portal for MUDs.")]
pub struct Args {
    #[arg(short, long, value_name = "ip:port", default_value = "127.0.0.1:7000", help = "Sets the (internal) link IpAddr and u16 port for IPC")]
    pub link: SocketAddr,

    #[arg(short, long, value_name = "ip:port", default_value = "0.0.0.0:1280", help = "Sets the external Telnet IpAddr and u16 port")]
    pub telnet: SocketAddr,

    #[arg(short, long, value_name = "ip:port", default_value = "0.0.0.0:8000", help = "Sets the external HTTP/WebSocket IpAddr and u16 port")]
    pub web: SocketAddr,

    #[arg(short, long, value_name = "path", help = "Sets the file path to a .pem file for TLS")]
    pub pem: Option<String>,

    #[arg(short, long, value_name = "path", help = "Sets the file path to a .key file for TLS")]
    pub key: Option<String>,
}

fn create_tls_acceptor(cert_path: &str, key_path: &str) -> Result<TlsAcceptor, Box<dyn std::error::Error>> {
    // Read the server certificate and chain
    let cert_file = std::fs::File::open(cert_path)?;
    let mut cert_reader = std::io::BufReader::new(cert_file);
    let cert_chain: Vec<Certificate> = certs(&mut cert_reader)?.into_iter().map(Certificate).collect();

    let key_file = std::fs::File::open(key_path)?;
    let mut key_reader = std::io::BufReader::new(key_file);
    let mut keys = pkcs8_private_keys(&mut key_reader)?;

    if keys.is_empty() {
        return Err("No private keys found".into());
    }
    let private_key = PrivateKey(keys.remove(0));

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)?;

    // Create the TLS acceptor
    let acceptor = TlsAcceptor::from(Arc::new(config));

    Ok(acceptor)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Thermite starting up...");

    let args: Args = Args::parse();

    let tls_acceptor = if args.pem.is_some() && args.key.is_some() {
        match create_tls_acceptor(args.pem.as_ref().unwrap(), args.key.as_ref().unwrap()) {
            Ok(tls_acceptor) => {
                *IS_TLS_ENABLED.lock().unwrap() = true;
                Some(Arc::new(tls_acceptor))
            }
            Err(e) => {
                error!("Error creating TLS acceptor: {}", e);
                None
            }
        }
    } else {
        None
    };


    let mut portal = Portal::new();

    *TX_PORTAL.lock().unwrap() = Some(portal.tx_portal.clone());

    let mut v = Vec::new();

    info!("Starting up networking...");
    info!("Starting up link acceptor on {}...", args.link);
    let mut link_acceptor = LinkAcceptor::new(args.link, portal.tx_portal.clone()).await?;
    v.push(tokio::spawn(async move {link_acceptor.run().await;}));
    info!("Starting up telnet acceptor on {}...", args.telnet);
    let mut telnet_acceptor = TelnetAcceptor::new(args.telnet, tls_acceptor.clone(), portal.tx_portal.clone()).await?;
    v.push(tokio::spawn(async move {telnet_acceptor.run().await;}));

    v.push(tokio::spawn(run_warp(args.web, args.pem, args.key)));

    info!("Starting up portal...");
    v.push(tokio::spawn(async move {portal.run().await;}));

    info!("Starting all tasks...");
    join_all(v).await;

    info!("Thermite shutting down.");
    Ok(())
}