use std::collections::{HashSet, HashMap};
use std::io::Error;

use crate::serialize::{RegisterRequest, SocketRequest};
use crate::store::{Client, Command, Store, Subscriptions, Subscribers};
use tokio::sync::mpsc::Sender;
use serde_json::{Value, json};
use warp::{Rejection, hyper::StatusCode};
use crate::Reply;
use crate::ws;
use crate::serialize::RequestAction;

pub async fn register_handler(body: RegisterRequest, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    // TODO: generate uuid and return to the client
    let user_id = body.user_id;
  
    Ok(json!({
        "url": format!("ws://127.0.0.1:8000/ws/{}", user_id),
    }).to_string())
  }

pub async fn unregister_handler(user_id: String, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    match Client::remove_client(user_id, clients_tx).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => Err(warp::reject::reject())
    }
}

pub async fn ws_handler(ws: warp::ws::Ws, user_id: String, subscribers_tx: Sender<Command<HashSet<Client>>>, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    let client = Client::get_client(user_id.clone(), clients_tx.clone()).await;

    match client {
        Ok(client) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, user_id, client, subscribers_tx, clients_tx))),
        Err(err) => Err(warp::reject::not_found())
    }
}

pub async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}

pub async fn ping_handler(user_id: &str, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    // todo: look up the user id and return a pong message
    Ok(StatusCode::OK)
}

pub async fn publish_handler(body: SocketRequest, user_id: String, store_tx: Sender<Command<String>>) -> Result<impl Reply, Rejection> {
    println!("publish request  from {}: {:?}", user_id, body);
    if let Some(message) = body.message {
        match body.action {
            RequestAction::Set => {
                Store::set(body.topic, message, store_tx).await;
                Ok(StatusCode::OK)
            },
            RequestAction::Remove => {
                Store::remove(body.topic, store_tx).await;
                Ok(StatusCode::OK)
            },
            _ => {
                eprintln!("Error: publish_handler must be called with a request of either Set or Remove");
                Err(warp::reject::reject())
            }
        }
    } else {
        Err(warp::reject::reject())
    }
    // TODO: add logic to alert the subscribers
}

pub async fn subscribe_handler(body: SocketRequest, user_id: String, subscribers_tx: Sender<Command<HashSet<Client>>>, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    println!("subscribe request  from {}: {:?}", user_id, body);
    let client = Client::get_client(user_id, clients_tx).await;
    if let Ok(Some(client)) = client {
        match body.action {
            RequestAction::Subscribe => {
                Subscribers::add_subscriber(body.topic, client, subscribers_tx);
                Ok(StatusCode::OK)
            },
            _ => {
                eprintln!("Error: subscribe_handler must be called with a request of Subscribe");
                Err(warp::reject::reject())
            }
        }
    } else {
        Err(warp::reject::reject())
    }
}

pub async fn unsubscribe_handler(body: SocketRequest, user_id: String, subscribers_tx: Sender<Command<HashSet<Client>>>, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    println!("unsubscribe request  from {}: {:?}", user_id, body);
    let client = Client::get_client(user_id, clients_tx).await;
    if let Ok(Some(client)) = client {
        match body.action {
            RequestAction::Subscribe => {
                Subscribers::add_subscriber(body.topic, client, subscribers_tx);
                Ok(StatusCode::OK)
            },
            _ => {
                eprintln!("Error: subscribe_handler must be called with a request of Subscribe");
                Err(warp::reject::reject())
            }
        }
    } else {
        Err(warp::reject::reject())
    }
}
