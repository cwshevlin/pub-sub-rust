use std::collections::HashSet;
use std::io::Error;
use std::{collections::HashMap, convert::Infallible};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, mpsc};
use warp::{Filter, Reply};

use crate::client::{Topics, Clients};
use crate::store::{Command};
mod frame;
mod client;
mod handler;
mod ws;
mod store;


#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let topics: Topics = Arc::new(Mutex::new(HashMap::new()));
    let (clients_tx, mut clients_rx) = mpsc::channel::<Result<Command, Error>>(32);
    let (topics_tx, mut topics_rx) = mpsc::channel::<Result<Command, Error>>(32);

    let health_route = warp::path!("health").and_then(handler::health_handler);
  
    let register_topics_tx = topics_tx.clone();
    let register_clients_tx = clients_tx.clone();
    let register = warp::path("register");
    let register_routes = register
      .and(warp::post())
      .and(warp::body::content_length_limit(1024 * 16))
      .and(warp::body::json())
      .and(with_clients(clients_tx))
      .and_then(handler::register_handler)
      .or(register
        .and(warp::delete())
        .and(warp::path::param())
        .and(with_clients(clients_tx))
        .and_then(handler::unregister_handler));
  
    let publish_topics_tx = topics_tx.clone();
    let publish_clients_tx = clients_tx.clone();
    let publish = warp::path("publish")
      .and(warp::post())
      .and(warp::body::content_length_limit(1024 * 16))
      .and(warp::body::json())
      .and(with_topics(topics_tx))
      .and(with_clients(clients_tx))
      .and_then(handler::publish_handler);

    let subscribe_topics_tx = topics_tx.clone();
    let subscribe_clients_tx = clients_tx.clone();
    let subscribe = warp::path("subscribe")
      .and(warp::post())
      .and(warp::body::content_length_limit(1024 * 16))
      .and(warp::body::json())
      .and(with_topics(topics_tx))
      .and(with_clients(clients_tx))
    .and_then(handler::subscribe_handler);

    let unsubscribe_topics_tx = topics_tx.clone();
    let unsubscribe_clients_tx = clients_tx.clone();
    let unsubscribe = warp::path("unsubscribe")
      .and(warp::post())
      .and(warp::body::content_length_limit(1024 * 16))
      .and(warp::body::json())
      .and(with_topics(topics_tx))
      .and(with_clients(clients_tx))
      .and_then(handler::unsubscribe_handler);
  
    let ws_route = warp::path("ws")
      .and(warp::ws())
      .and(warp::path::param())
      .and(with_topics(topics_tx))
      .and(with_clients(clients_tx))
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

fn with_clients(clients_tx: Sender<Result<Command, Error>>) -> impl Filter<Extract = (Sender<Result<Command, Error>>,), Error = Infallible> + Clone {
    warp::any().map(move || clients_tx.clone())
}

fn with_topics(topics_tx: Sender<Result<Command, Error>>) -> impl Filter<Extract = (Sender<Result<Command, Error>>,), Error = Infallible> + Clone {
    warp::any().map(move || topics_tx.clone())
}
