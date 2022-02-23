use std::io::Error;

use crate::client::{Event, RegisterRequest, SubscribeRequest, UnsubscribeRequest};
use crate::store::{Client, Command};
use tokio::sync::mpsc::Sender;
use serde_json::{Value, json};
use warp::{Rejection, hyper::StatusCode};
use crate::Reply;
use crate::ws;

pub async fn register_handler(body: RegisterRequest, clients_tx: Sender<Command<Option<Client>>>) -> Result<impl Reply, Rejection> {
    let user_id = body.user_id;
    // let uuid = Uuid::new_v4().to_string();
    // TODO CWS: generate new uuids
    let uuid = String::from("cbf99b28-4488-45c9-aa12-46b3cfb979bb");
  
    match register_client(user_id.clone(), clients_tx).await {
        Ok(_) => {
            return Ok(json!({
            "url": format!("ws://127.0.0.1:8000/ws/{}", user_id),
            }).to_string())
        },
        Err(_) => Err(warp::reject::reject())
    }
  }
    
async fn register_client(user_id: String, clients_tx: Sender<ClientsCommand>) -> Result<impl Reply, Rejection> {
    insert_client(Client { user_id: user_id, sender: None }, clients_tx.clone()).await;
    Ok(StatusCode::OK)
}

pub async fn unregister_handler(user_id: String, clients_tx: Sender<ClientsCommand>) -> Result<impl Reply, Rejection> {
    remove_client(user_id, clients_tx.clone()).await;
    Ok(StatusCode::OK)
}

pub async fn ws_handler(ws: warp::ws::Ws, user_id: String, subscribers_tx: Sender<SubscribersCommand>, clients_tx: Sender<ClientsCommand>) -> Result<impl Reply, Rejection> {
    let client = get_client(user_id.clone(), clients_tx.clone()).await;

    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, user_id, c, subscribers_tx, clients_tx))),
        None => Err(warp::reject::not_found())
    }
}

pub async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}

pub async fn publish_handler(body: Event, subscribers_tx: Sender<SubscribersCommand>, clients_tx: Sender<ClientsCommand>) -> Result<impl Reply, Rejection> {
    todo!();
}

pub async fn subscribe_handler(body: SubscribeRequest, subscribers_tx: Sender<SubscribersCommand>, clients_tx: Sender<ClientsCommand>) -> Result<impl Reply, Rejection> {
    todo!();
}

pub async fn unsubscribe_handler(body: UnsubscribeRequest, subscribers_tx: Sender<SubscribersCommand>, clients_tx: Sender<ClientsCommand>) -> Result<impl Reply, Rejection> {
    todo!();
}
