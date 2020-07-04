//use actix_rt::{main};
use actix::prelude::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use thermite::net::{NetworkManager};
use thermite::telnet::{TelnetActor, TcpServer};
use actix_web::client::ClientBuilder;
use tokio::net::TcpListener;
use std::net::TcpListener as StdListener;

#[actix_rt::main]
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
