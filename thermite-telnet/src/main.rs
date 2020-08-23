use tokio::prelude::*;
use thermite_telnet::telnet::{TelnetServer};
use tokio::net::TcpListener;


#[tokio::main]
async fn main() {

    let mut listener = TcpListener::bind("0.0.0.0:7999").await.unwrap();
    let mut srv = TelnetServer::new();

    srv.listen(String::from("telnet"), listener, None);

    srv.run().await;

}