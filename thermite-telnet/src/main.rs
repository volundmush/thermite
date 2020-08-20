use tokio::prelude::*;
use crate::net::{NetworkManager};

use tokio::net::TcpListener;


#[tokio::main]
async fn main() -> Result<(), Error> {

    let mut net_man = NetworkManager::default().start();
    
    let mut listener = StdListener::bind("0.0.0.0:7999").unwrap();
    let mut listener = TcpListener::from_std(listener).unwrap();
    
    let mut telnet_srv = TcpServer::create(|ctx| {
        ctx.add_stream(listener);
        TcpServer {
            connections: Default::default(),
            manager: net_man.clone()
        }
    });

    HttpServer::new(move || {
        App::new()
            .data(net_man.clone())
    })
        .bind("0.0.0.0:8000")?
        .run()
        .await;
    Ok(())
}
