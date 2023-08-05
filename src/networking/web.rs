use warp::{Filter, Reply};
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use once_cell::sync::Lazy;

use crate::{
    protocols::websocket::protocol::WebsocketProtocol,
    networking::CONNECTION_ID_COUNTER,
    IS_TLS_ENABLED,
    util::resolve_hostname
};

static TERA: Lazy<tera::Tera> = Lazy::new(|| {
    let mut tera = match tera::Tera::new("webroot/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    tera
});

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

    let http_static = warp::path("static")
        .and(warp::fs::dir("webroot/static"));

    let wclient = warp::path::end().map(|| {
        let mut context = tera::Context::new();
        context.insert("game_name", "Thermite Webclient");
        context.insert("websocket_enabled", "true");
        context.insert("websocket_port", "8000");
        let response: Box<dyn warp::Reply> = match TERA.render("webclient/webclient.html", &context) {
            Ok(rendered) => Box::new(warp::reply::html(rendered)),
            Err(err) => {
                eprintln!("Template rendering error: {}", err);
                Box::new(warp::reply::with_status(
                    "Internal server error",
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        };
        response
    });

    let log = warp::log("example::api");

    // Combine routes
    let routes = ws_route
        .or(http_static)
        .or(wclient)
        .with(log);

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