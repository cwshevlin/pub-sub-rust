use std::{collections::HashMap, convert::Infallible};
use std::io::Error;
use std::sync::Arc;
use client::Topics;
use tokio::sync::{Mutex, mpsc};
use warp::{Filter, Rejection, Reply};
mod frame;
mod client;
mod handler;
mod ws;

/// Multiple different commands are multiplexed over a single channel.
// #[derive(Debug)]
type Result<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    let clients: client::Clients = Arc::new(Mutex::new(HashMap::new()));

    let health_route = warp::path!("health").and_then(handler::health_handler);
  
    let register = warp::path("register");
    let register_routes = register
      .and(warp::post())
      .and(warp::body::content_length_limit(1024 * 16))
      .and(warp::body::json())
      .and(with_clients(clients.clone()))
      .and_then(handler::register_handler)
      .or(register
        .and(warp::delete())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(handler::unregister_handler));
  
    let publish = warp::path("publish")
      .and(warp::post())
      .and(warp::body::content_length_limit(1024 * 16))
      .and(warp::body::json())
      .and(with_clients(clients.clone()))
      .and_then(handler::publish_handler);
  
    let ws_route = warp::path("ws")
      .and(warp::ws())
      .and(warp::path::param())
      .and(with_clients(clients.clone()))
      .and_then(handler::ws_handler);
  
    let routes = health_route
      .or(register_routes)
      .or(ws_route)
      .or(publish)
      .with(warp::cors().allow_any_origin());
  
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_clients(clients: client::Clients) -> impl Filter<Extract = (client::Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
