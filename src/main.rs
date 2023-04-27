use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    path::{PathBuf}
};
use tokio::select;
use tokio::sync::mpsc::{Sender, Receiver, channel};
use clap::{App, Arg};

use thermite::{
    networking::{
        internal::InternalAcceptor,
        external::ExternalAcceptor
    }
};
use thermite::msg::Msg2Portal;
use thermite::portal::Portal;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Thermite")
        .version("0.1")
        .author("Andrew Bastien <volundmush@gmail.com>")
        .about("A networking portal for MUDs.")
        .arg(
            Arg::new("i")
                .long("i")
                .value_name("ip:port")
                .about("Sets the internal IpAddr and u16 port for IPC")
                .default_value("127.0.0.1:7000")
                .takes_value(true),
        )
        .arg(
            Arg::new("e")
                .long("e")
                .value_name("ip:port")
                .about("Sets the external IpAddr and u16 port")
                .default_value("0.0.0.0:7999")
                .takes_value(true),
        )
        .arg(
            Arg::new("pem")
                .long("pem")
                .value_name("path")
                .about("Sets the file path to a .pem file for TLS")
                .takes_value(true),
        )
        .arg(
            Arg::new("key")
                .long("key")
                .value_name("path")
                .about("Sets the file path to a .key file for TLS")
                .takes_value(true),
        )
        .get_matches();

    let internal_addr: SocketAddr = matches.value_of("i")?.parse()?;
    let external_addr: SocketAddr = matches.value_of("e")?.parse()?;
    let pem_path: Option<PathBuf> = matches.value_of("pem").map(PathBuf::from);
    let key_path: Option<PathBuf> = matches.value_of("key").map(PathBuf::from);

    let mut internal_acceptor = InternalAcceptor::new(internal_addr);
    let mut portal = Portal::new();
    let mut external_acceptor = ExternalAcceptor::new(external_addr, pem_path, key_path, portal.tx_portal.clone());

    select! {
        _ = internal_acceptor.run() => {},
        _ = external_acceptor.run() => {}
    }

    Ok(())
}