use warp::{Filter, Reply};
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use trust_dns_resolver::TokioAsyncResolver;
use warp::filters::BoxedFilter;
use crate::{
    protocols::websocket::protocol::WebsocketProtocol
};
use warp::{serve, Server, TlsServer};
use crate::{
    networking::CONNECTION_ID_COUNTER,
    IS_TLS_ENABLED,
    util::resolve_hostname
};

async fn handle_websocket(ws: warp::ws::WebSocket, addr: Option<SocketAddr>) {
    // Create the actor with the WebSocket
    let mut hostnames = Vec::new();
    let mut ip = String::from("0.0.0.0");
    let mut port: u16 = 0;
    if let Some(a) = addr {
        ip = a.ip().to_string();
        port = a.port();
        if let Ok(h) = resolve_hostname(a).await {
            hostnames = h;
        }
    }

    let conn_id = CONNECTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut prot = WebsocketProtocol::new(conn_id, ws, ip, port, hostnames);

    // Run the actor
    let _ = prot.run().await;

}


pub async fn run_warp(addr: SocketAddr, pem: Option<String>, key: Option<String>) {
    // WebSocket route
    let ws_route = warp::path("ws")
        .and(warp::addr::remote()) // Get the remote address
        .and(warp::ws())
        .map(|remote_addr: Option<SocketAddr>, ws: warp::ws::Ws| {
            // You can now access the remote address inside this closure
            ws.on_upgrade(move |websocket | handle_websocket(websocket, remote_addr))
        });

    // Other HTTP routes...
    let http_routes = warp::any()
        .map(|| "Hello, HTTP!");


    // Combine routes
    let routes = ws_route.or(http_routes);

    let mut warp = warp::serve(routes);

    if *IS_TLS_ENABLED.lock().unwrap() {
        let mut warptls = warp.tls()
            .key_path(key.unwrap())
            .cert_path(pem.unwrap());
        let _ = warptls.bind(addr).await;
    } else {
        let _ = warp.bind(addr).await;
    }
}