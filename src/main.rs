use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
};

use std::sync::Arc;
use clap::{Parser};
use futures::future::join_all;

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


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Thermite starting up...");

    let args: Args = Args::parse();

    let mut portal = Portal::new();

    *TX_PORTAL.lock().unwrap() = Some(portal.tx_portal.clone());

    let mut v = Vec::new();

    info!("Starting up networking...");
    info!("Starting up link acceptor on {}...", args.link);
    let mut link_acceptor = LinkAcceptor::new(args.link, portal.tx_portal.clone()).await?;
    v.push(tokio::spawn(async move {link_acceptor.run().await;}));
    info!("Starting up telnet acceptor on {}...", args.telnet);
    let mut telnet_acceptor = TelnetAcceptor::new(args.telnet, portal.tx_portal.clone()).await?;
    v.push(tokio::spawn(async move {telnet_acceptor.run().await;}));

    v.push(tokio::spawn(run_warp(args.web, args.pem, args.key)));

    info!("Starting up portal...");
    v.push(tokio::spawn(async move {portal.run().await;}));

    info!("Starting all tasks...");
    join_all(v).await;

    info!("Thermite shutting down.");
    Ok(())
}