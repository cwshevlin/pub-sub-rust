use std::collections::HashSet;
use std::{collections::HashMap, convert::Infallible};
use std::sync::Arc;
use tokio::sync::{Mutex};
use warp::{Filter, Reply};

use crate::client::{Topics, Clients};
mod frame;
mod client;
mod handler;
mod ws;


#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let topics: Topics = Arc::new(Mutex::new(HashMap::new()));

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
      .and(with_topics(topics.clone()))
      .and(with_clients(clients.clone()))
      .and_then(handler::publish_handler);

    let subscribe = warp::path("subscribe")
      .and(warp::post())
      .and(warp::body::content_length_limit(1024 * 16))
      .and(warp::body::json())
      .and(with_topics(topics.clone()))
      .and(with_clients(clients.clone()))
    .and_then(handler::subscribe_handler);

    let unsubscribe = warp::path("unsubscribe")
      .and(warp::post())
      .and(warp::body::content_length_limit(1024 * 16))
      .and(warp::body::json())
      .and(with_topics(topics.clone()))
      .and(with_clients(clients.clone()))
      .and_then(handler::unsubscribe_handler);
  
    let ws_route = warp::path("ws")
      .and(warp::ws())
      .and(warp::path::param())
      .and(with_topics(topics.clone()))
      .and(with_clients(clients.clone()))
      .and_then(handler::ws_handler);
  
    let routes = health_route
      .or(register_routes)
      .or(ws_route)
      .or(publish)
      .or(subscribe)
      .or(unsubscribe)
      .with(warp::cors().allow_any_origin());
  
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_topics(topics: Topics) -> impl Filter<Extract = (Topics,), Error = Infallible> + Clone {
    warp::any().map(move || topics.clone())
}
