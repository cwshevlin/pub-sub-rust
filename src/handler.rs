use std::collections::{HashSet};
use crate::serialize::{RegisterRequest, SocketRequest};
use crate::store::{Client, Command, Store, Subscribers, Subscriptions};
use tokio::sync::mpsc::Sender;
use serde_json::{json};
use warp::ws::Message;
use warp::{Rejection, hyper::StatusCode};
use crate::Reply;
use crate::ws;
use crate::serialize::RequestAction;
use log::{info, trace, warn};

pub async fn register_handler(body: RegisterRequest, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    // TODO: generate uuid and return to the client
    let user_id = body.user_id;
    let client = Client {
        user_id: user_id.clone(),
        sender: None
    };
    match Client::set_client(client, clients_tx.clone()).await {
        Ok(_) => {
            Ok(json!({
                "url": format!("ws://127.0.0.1:8000/ws/{}", user_id),
            }).to_string())
        },
        Err(_) => Err(warp::reject::reject())
    }
  }

pub async fn unregister_handler(user_id: String, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    match Client::remove_client(user_id, clients_tx).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(warp::reject::reject())
    }
}

pub async fn ws_handler(ws: warp::ws::Ws, user_id: String, subscriptions_tx: Sender<Command<HashSet<Client>>>, clients_tx: Sender<Command<Client>>, store_tx: Sender<Command<String>>) -> Result<impl Reply, Rejection> {
    let client = Client::get_client(user_id.clone(), clients_tx.clone()).await;
    println!(" client: {:?}", client);

    match client {
        Ok(Some(client)) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, user_id, client, subscriptions_tx, clients_tx, store_tx))),
        _ => Err(warp::reject::not_found())
    }
}

pub async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}

pub async fn ping_handler(user_id: &str, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    // todo: look up the user id and return a pong message
    Ok(StatusCode::OK)
}

pub async fn publish_handler(body: SocketRequest, user_id: String, subscriptions_tx: Sender<Command<HashSet<Client>>>, clients_tx: Sender<Command<Client>>, store_tx: Sender<Command<String>>) -> Result<impl Reply, Rejection> {
    println!("publish request  from {}: {:?}", user_id, body);
    if let Some(message) = body.message {
        match body.action {
            RequestAction::Set => {
                match Store::set(body.topic.clone(), message.clone(), store_tx).await {
                    Ok(_) => alert_subscribers(body.topic, message, subscriptions_tx).await,
                    Err(_) => Err(warp::reject::reject())
                }
            },
            RequestAction::Remove => {
                match Store::remove(body.topic, store_tx).await {
                    Ok(_) => Ok(StatusCode::OK),
                    Err(_) => Err(warp::reject::reject())
                }
            },
            _ => {
                eprintln!("Error: publish_handler must be called with a request of either Set or Remove");
                Err(warp::reject::reject())
            }
        }
    } else {
        Err(warp::reject::reject())
    }
}

async fn alert_subscribers(topic: String, value: String, subscriptions_tx: Sender<Command<HashSet<Client>>>) -> Result<StatusCode, Rejection> {
    match Subscribers::get_subscribers(topic.clone(), subscriptions_tx).await {
        Ok(Some(subscribers)) => {
            for client in subscribers {
                match client.sender {
                    Some(sender) => {
                        match sender.send(Ok(Message::text(value.clone()))) {
                            Ok(_) => info!("Subscriber alerted: {:?}", &client.user_id),
                            Err(_) => warn!("Error sending update to subscriber: {:?}", &client.user_id)
                        }
                    }
                    None => {
                        warn!("Sender not found on client {}", &client.user_id)
                    }
                }
            }
            Ok(StatusCode::OK)
        },
        Ok(None) => {
            info!("No clients found subscribed to topic {}, skipping", topic.clone());
            Ok(StatusCode::OK)
        }
        Err(_) => Err(warp::reject::reject())
    }
}

pub async fn subscribe_handler(body: SocketRequest, user_id: String, subscriptions_tx: Sender<Command<HashSet<Client>>>, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    println!("subscribe request  from {}: {:?}", user_id, body);
    // TODO CWS: use match here instead of the conditional
    let client = Client::get_client(user_id, clients_tx).await;
    if let Ok(Some(client)) = client {
        match body.action {
            RequestAction::Subscribe => {
                match Subscribers::add_subscriber(body.topic, client, subscriptions_tx).await {
                    Ok(_) => Ok(StatusCode::OK),
                    Err(_) => Err(warp::reject::reject())
                }
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

pub async fn unsubscribe_handler(body: SocketRequest, user_id: String, subscriptions_tx: Sender<Command<HashSet<Client>>>, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    println!("unsubscribe request  from {}: {:?}", user_id, body);
    let client = Client::get_client(user_id, clients_tx).await;
    if let Ok(Some(client)) = client {
        match body.action {
            RequestAction::Subscribe => {
                match Subscribers::add_subscriber(body.topic, client, subscriptions_tx).await {
                    Ok(_) => Ok(StatusCode::OK),
                    Err(_) => Err(warp::reject::reject())
                }
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
