use std::collections::HashSet;
use std::{collections::HashMap, convert::Infallible};
use std::sync::Arc;
use store::{Command, Client};
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, mpsc};
use warp::{Filter, Reply};
use crate::store::{Subscriptions, Clients};
mod serialize;
mod handler;
mod ws;
mod store;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
  env_logger::init();

  let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
  let subscriptions: Subscriptions = Arc::new(Mutex::new(HashMap::new()));
  let store = Arc::new(Mutex::new(HashMap::new()));

  let (clients_tx, mut clients_rx) = mpsc::channel::<Command<Client>>(32);
  let (subscriptions_tx, mut subscriptions_rx) = mpsc::channel::<Command<HashSet<Client>>>(32);
  let (store_tx, mut store_rx) = mpsc::channel::<Command<String>>(32);
  
  // TODO CWS: move this and other similar logic to the store implementations?
  tokio::spawn(async move {
    while let Some(cmd) = clients_rx.recv().await {
      // TODO: pass the data structure here so that it is the only one that has access?
        match cmd {
            Command::Get { key, responder } => {
                  info!("Get from client store: {:?}", key);
                if let Some(result) = clients.lock().await.get(&key) {
                  // TODO CWS: this clone is probably unecessary. What can we do with references here?
                  println!("Result from client store: {:?}", result);
                  let _ = responder.send(Some(result.clone()));
                }
            }
            Command::Set { key, value, responder } => {
                let result = clients.lock().await.insert(key, value);
                let _ = responder.send(result);
            }
            Command::Remove { key, responder } => {
                let result = clients.lock().await.remove(&key);
                let _ = responder.send(result);
            }
        }
    }
  });

  tokio::spawn(async move {
    while let Some(cmd) = subscriptions_rx.recv().await {
      // TODO: pass the data structure here so that it is the only one that has access?
        match cmd {
            Command::Get { key, responder } => {
                if let Some(result) = subscriptions.lock().await.get(&key) {
                  // TODO CWS: this clone is probably unecessary. What can we do with references here?
                  let _ = responder.send(Some(result.clone()));
                }
            }
            Command::Set { key, value, responder } => {
                let result = subscriptions.lock().await.insert(key, value);
                let _ = responder.send(result);
            }
            Command::Remove { key, responder } => {
                let result = subscriptions.lock().await.remove(&key);
                let _ = responder.send(result);
            }
        }
    }
  });

  tokio::spawn(async move {
    while let Some(cmd) = store_rx.recv().await {
      // TODO: pass the data structure here so that it is the only one that has access?
        match cmd {
            Command::Get { key, responder } => {
                if let Some(result) = store.lock().await.get(&key) {
                  let _ = responder.send(Some(String::from(result)));
                }
            }
            Command::Set { key, value, responder } => {
                let result = store.lock().await.insert(key, value);
                let _ = responder.send(result);
            }
            Command::Remove { key, responder } => {
                let result = store.lock().await.remove(&key);
                let _ = responder.send(result);
            }
        }
    }
  });
  


  let health_route = warp::path!("health").and_then(handler::health_handler);

  let register = warp::path("register");
  let register_routes = register
    .and(warp::post())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(warp::body::json())
    .and(with_clients(clients_tx.clone()))
    .and_then(handler::register_handler)
    .or(register
      .and(warp::delete())
      .and(warp::path::param())
      .and(with_clients(clients_tx.clone()))
      .and_then(handler::unregister_handler));

  let ws_route = warp::path("ws")
    .and(warp::ws())
    .and(warp::path::param())
    .and(with_subscriptions(subscriptions_tx))
    .and(with_clients(clients_tx.clone()))
    .and(with_store(store_tx))
    .and_then(handler::ws_handler);

  let routes = health_route
    .or(register_routes)
    .or(ws_route)
    .with(warp::cors().allow_any_origin());

  println!("Server started");
  warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_clients(clients_tx: Sender<Command<Client>>) -> impl Filter<Extract = (Sender<Command<Client>>,), Error = Infallible> + Clone {
    warp::any().map(move || clients_tx.clone())
}

fn with_subscriptions(subscriptions_tx: Sender<Command<HashSet<Client>>>) -> impl Filter<Extract = (Sender<Command<HashSet<Client>>>,), Error = Infallible> + Clone {
    warp::any().map(move || subscriptions_tx.clone())
}

fn with_store(store_tx: Sender<Command<String>>) -> impl Filter<Extract = (Sender<Command<String>>,), Error = Infallible> + Clone {
    warp::any().map(move || store_tx.clone())
}