use tokio::prelude::*;
use thermite_net::{NetworkManager, TxToNetManager, ServerDef, TCPTransport, TelnetProtocol, TelnetHandler};
use tokio::time::{delay_for, Instant, Duration};

#[tokio::main]
async fn main() {

    let mut net = NetworkManager::default();
    // Yoinking our controls for the NetworkManager!

    net.create_server(ServerDef {
        name: String::from("echo"),
        address: String::from("10.0.0.226"),
        port: 4200
    },
    );

    let mut tx_net = net.tx_master.clone();

    let mut handle = tokio::spawn(async move {
        net.run().await;
    });

    tx_net.send(TxToNetManager::CreateServer());

    handle.await;

    println!("It's amazing we got this far.");
}
