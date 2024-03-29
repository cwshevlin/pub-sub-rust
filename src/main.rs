use std::collections::HashSet;
use std::{collections::HashMap, convert::Infallible};
use std::sync::Arc;
use store::Client;
use command::{Command};
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, mpsc};
use warp::{Filter, Reply};
use crate::store::{Subscriptions, Clients};
mod serialize;
mod handler;
mod ws;
mod store;
mod command;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
  env_logger::init();

  // TODO CWS: I wonder if this combination of Arc/Mutex is the right approach or if we could do this pattern with just an Arc and moves.
  let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
  let subscriptions: Subscriptions = Arc::new(Mutex::new(HashMap::new()));
  let string_store = Arc::new(Mutex::new(HashMap::new()));
  let collection_store = Arc::new(Mutex::new(HashMap::<String, HashSet::<String>>::new()));

  let (clients_tx, mut clients_rx) = mpsc::channel::<Command<Client>>(32);
  let (subscriptions_tx, mut subscriptions_rx) = mpsc::channel::<Command<Client>>(32);
  let (store_tx, mut store_rx) = mpsc::channel::<Command<String>>(32);
  
  // TODO CWS: move this and other similar logic to the store implementations?
  tokio::spawn(async move {
    while let Some(cmd) = clients_rx.recv().await {
      // TODO: pass the data structure here so that it is the only one that has access?
        match cmd {
            Command::GetItem { key, responder } => {
                info!("Get from client store: {:?}", key);
                if let Some(result) = clients.lock().await.get(&key) {
                  // TODO CWS: this clone is probably unecessary. What can we do with references here? And if we do that, can we include referenced variables in the logs?
                  let _ = responder.send(Some(result.clone()));
                }
            },
            Command::SetItem { key, value, responder } => {
                info!("Set value: {:?} for key: {:?} in the client store.", value, key);
                let result = clients.lock().await.insert(key, value);
                let _ = responder.send(result);
            },
            Command::UnsetItem { key, responder } => {
                info!("Unset key: {:?} in the client store.", key);
                let result = clients.lock().await.remove(&key);
                let _ = responder.send(result);
            },
            _ => {
                error!("Only Get, Set and Unset may be used with clients.");
            }
        }
    }
  });

  tokio::spawn(async move {
    while let Some(cmd) = subscriptions_rx.recv().await {
      // TODO: pass the data structure here so that it is the only one that has access?
        match cmd {
            Command::GetCollection { key, responder } => {
                if let Some(result) = subscriptions.lock().await.get(&key) {
                  info!("Get key {:?} from the subscriptions store. Result: {:?}", key, result);
                  // TODO CWS: this clone is probably unecessary. What can we do with references here?
                  let _ = responder.send(Some(result.clone()));
                }
            },
            Command::RemoveFromCollection { key, value, responder } => {
                let mut subscriptions = subscriptions.lock().await;
                let subscriptions_option = subscriptions.get_mut(&key);
                let result = match subscriptions_option {
                  Some(collection) => collection.remove(&value),
                  None => false
                };
                info!("Remove key {:?} from the subscriptions store. Result: {:?}", key, result);
                let _ = responder.send(result);
            },
            Command::AddToCollection { key, value, responder } => {
                let mut subscriptions = subscriptions.lock().await;
                let subscriptions_option = subscriptions.get_mut(&key);
                let result = match subscriptions_option {
                  Some(subscription) => subscription.insert(value),
                  None => {
                    let mut subscription = HashSet::new();
                    subscription.insert(value);
                    let insert_result = subscriptions.insert(key, subscription);
                    if insert_result.is_some() {
                      false
                    } else {
                      true
                    }
                  }
                };
                info!("Add to collection in the subscriptions store. Result: {:?}", result);
                let _ = responder.send(result);
            }
            _ => {
                error!("Only Get, Set and Unset may be used with subscriptions.");
            }
        }
    }
  });

  tokio::spawn(async move {
    while let Some(cmd) = store_rx.recv().await {
      // TODO: pass the data structure here so that it is the only one that has access?
        match cmd {
            Command::GetItem { key, responder } => {
                if let Some(result) = string_store.lock().await.get(&key) {
                  info!("Get key {:?} in the string store. Result: {:?}", key, result);
                  let _ = responder.send(Some(String::from(result)));
                }
            },
            Command::SetItem { key, value, responder } => {
                let result = string_store.lock().await.insert(key, value);
                info!("Set key in the string store. Result: {:?}. Current store: {:?}", result, string_store.lock().await);
                let _ = responder.send(result);
            },
            Command::UnsetItem { key, responder } => {
                let result = string_store.lock().await.remove(&key);
                info!("Unset key in the string store. Key: {:?}, Result: {:?}, Current store: {:?}", key, result, string_store.lock().await);
                let _ = responder.send(result);
            },
            Command::RemoveFromCollection { key, value, responder } => {
                let mut collection_store = collection_store.lock().await;
                let collection_option = collection_store.get_mut(&key);
                let result = match collection_option {
                  Some(collection) => collection.remove(&value),
                  None => false
                };
                info!("Remove from collection in the string store. Key: {:?}, Value: {:?} Result: {:?}, Current store: {:?}", key, value, result, string_store.lock().await);
                let _ = responder.send(result);
            },
            Command::AddToCollection { key, value, responder } => {
                let mut collection_store = collection_store.lock().await;
                let collection_option = collection_store.get_mut(&key);
                let result = match collection_option {
                  Some(collection) => collection.insert(value),
                  None => false
                };
                info!("Add to collection in the string store. Key: {:?}, Result: {:?}, Current store: {:?}", key, result, string_store.lock().await);
                let _ = responder.send(result);
            }
            Command::GetCollection { key, responder } => {
                let collection_store = collection_store.lock().await;
                let collection_option = collection_store.get(&key);
                let result = match collection_option {
                    Some(collection) => Some(collection.clone()),
                    None => None
                };
                info!("Get collection in the string store. Key: {:?}, Result: {:?}, Current store: {:?}", key, result, string_store.lock().await);
                let _ = responder.send(result);
            }
        }
    }
  });

  let health_route = warp::path!("health").and_then(handler::health_handler);

  let register = warp::path("register");
  let register_routes = register
    .and(warp::get())
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

fn with_subscriptions(subscriptions_tx: Sender<Command<Client>>) -> impl Filter<Extract = (Sender<Command<Client>>,), Error = Infallible> + Clone {
    warp::any().map(move || subscriptions_tx.clone())
}

fn with_store(store_tx: Sender<Command<String>>) -> impl Filter<Extract = (Sender<Command<String>>,), Error = Infallible> + Clone {
    warp::any().map(move || store_tx.clone())
}