use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    path::{PathBuf}
};

use std::sync::Arc;
use tokio::runtime::Builder;
use tokio::select;
use tokio::sync::mpsc::{Sender, Receiver, channel};
use clap::{Parser, Arg};
use futures::future::join_all;
use tokio::fs::File;
use tokio::io::BufReader;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::rustls::server::NoClientAuth;
use tokio_rustls::TlsAcceptor;

use tracing::{info, Level};
use tracing_subscriber;

use thermite::{
    networking::{
        link::LinkAcceptor,
        telnet::TelnetAcceptor
    }
};
use thermite::msg::Msg2Portal;
use thermite::portal::Portal;

#[derive(Parser, Debug)]
#[clap(version = "0.1", author = "Andrew Bastien <volundmush@gmail.com>", about = "A networking portal for MUDs.")]
pub struct Args {
    #[arg(short, long, value_name = "ip:port", default_value = "127.0.0.1:7000", help = "Sets the (internal) link IpAddr and u16 port for IPC")]
    pub link: SocketAddr,

    #[arg(short, long, value_name = "ip:port", default_value = "0.0.0.0:7999", help = "Sets the external Telnet IpAddr and u16 port")]
    pub telnet: SocketAddr,

    #[arg(short, long, value_name = "ip:port", default_value = "0.0.0.0:8000", help = "Sets the external HTTP/WebSocket IpAddr and u16 port")]
    pub web: SocketAddr,

    #[arg(short, long, value_name = "path", help = "Sets the file path to a .pem file for TLS")]
    pub pem: Option<String>,

    #[arg(short, long, value_name = "path", help = "Sets the file path to a .key file for TLS")]
    pub key: Option<String>,
}


async fn run() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Thermite starting up...");

    let args: Args = Args::parse();

    let tls_acceptor = if args.pem.is_some() && args.key.is_some() {
        None
    } else {
        None
    };
    //info!("TLS Acceptor: {:?}", tls_acceptor);


    let mut portal = Portal::new();

    let mut v = Vec::new();

    info!("Starting up networking...");
    info!("Starting up link acceptor on {}...", args.link);
    let mut link_acceptor = LinkAcceptor::new(args.link, portal.tx_portal.clone()).await?;
    let link_join = tokio::spawn(async move {
        link_acceptor.run().await
    });
    v.push(link_join);
    info!("Starting up telnet acceptor on {}...", args.telnet);
    let mut telnet_acceptor = TelnetAcceptor::new(args.telnet, tls_acceptor.clone(), portal.tx_portal.clone()).await?;
    let telnet_join = tokio::spawn(async move {
        telnet_acceptor.run().await
    });
    v.push(telnet_join);


    info!("Starting up portal...");
    let portal_join = tokio::spawn(async move {
        portal.run().await
    });
    v.push(portal_join);

    info!("Starting all tasks...");
    join_all(v).await;

    info!("Thermite shutting down.");
    Ok(())
}

fn main() {
    let runtime = Builder::new_multi_thread()
        .thread_stack_size(12 * 1024 * 1024) // Set the stack size for each worker thread to 4 MB
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(run());
}