use crate::serialize::{SocketRequest};
use crate::store::{Client, Store, Subscribers};
use crate::command::Command;
use tokio::sync::mpsc::Sender;
use serde_json::{json};
use warp::ws::Message;
use warp::{Rejection, hyper::StatusCode};
use crate::Reply;
use crate::ws;
use crate::serialize::RequestAction;
use log::{warn, error};
use uuid::Uuid;

pub async fn register_handler(clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    // TODO: generate uuid and return to the client
    let user_id = Uuid::new_v4();
    println!("Registering: {}", user_id.to_string().clone());
    // TODO: does this sender need to be populated?
    let client = Client {
        user_id: user_id.to_string(),
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

pub async fn ws_handler(ws: warp::ws::Ws, user_id: String, subscriptions_tx: Sender<Command<Client>>, clients_tx: Sender<Command<Client>>, store_tx: Sender<Command<String>>) -> Result<impl Reply, Rejection> {
    println!("ws handler: {}", user_id.to_string().clone());
    let client = Client::get_client(user_id.clone(), clients_tx.clone()).await;

    match client {
        Ok(Some(client)) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, user_id, client, subscriptions_tx, clients_tx, store_tx))),
        _ => Err(warp::reject::not_found())
    }
}

pub async fn health_handler() -> Result<StatusCode, Rejection> {
    Ok(StatusCode::OK)
}

pub async fn publish_handler(body: SocketRequest, user_id: String, subscriptions_tx: Sender<Command<Client>>, store_tx: Sender<Command<String>>) -> Result<impl Reply, Rejection> {
    if let Some(message) = body.message {
        match body.action {
            RequestAction::Set => {
                match Store::set(body.topic.clone(), message.clone(), store_tx).await {
                    Ok(_) => alert_subscribers(body.topic, message, user_id, subscriptions_tx).await,
                    Err(_) => Err(warp::reject::reject())
                }
            },
            RequestAction::Unset => {
                match Store::unset(body.topic.clone(), store_tx).await {
                    Ok(_) => alert_subscribers(body.topic, String::from(""), user_id, subscriptions_tx).await,
                    Err(_) => Err(warp::reject::reject())
                }
            },
            RequestAction::AddToCollection => {
                match Store::add_to_collection(body.topic, message.clone(), store_tx).await {
                    Ok(_) => Ok(StatusCode::OK),
                    Err(_) => Err(warp::reject::reject())
                }
            },
            RequestAction::RemoveFromCollection => {
                match Store::remove_value_from_collection(body.topic, message.clone(), store_tx).await {
                    Ok(_) => Ok(StatusCode::OK),
                    Err(_) => Err(warp::reject::reject())
                }
            },
            _ => {
                error!("Error: publish_handler must be called with a request of either Set, Remove, or Add");
                Err(warp::reject::reject())
            }
        }
    } else {
        Err(warp::reject::reject())
    }
}

async fn alert_subscribers(topic: String, value: String, user_id: String, subscriptions_tx: Sender<Command<Client>>) -> Result<StatusCode, Rejection> {
    match Subscribers::get_subscribers(topic.clone(), subscriptions_tx).await {
        Ok(Some(subscribers)) => {
            for client in subscribers {
                if client.user_id == user_id {
                    continue;
                }
                match client.sender {
                    Some(sender) => {
                        match sender.send(Ok(Message::text(value.clone()))) {
                            Ok(_) => debug!("Subscriber alerted: {:?}", &client.user_id),
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
            debug!("No clients found subscribed to topic {}, skipping", topic.clone());
            Ok(StatusCode::OK)
        }
        Err(_) => {
            error!("Error getting subscribers.");
            Err(warp::reject::reject())
        }
    }
}

pub async fn subscription_handler(body: SocketRequest, user_id: String, subscriptions_tx: Sender<Command<Client>>, clients_tx: Sender<Command<Client>>) -> Result<impl Reply, Rejection> {
    let client = Client::get_client(user_id, clients_tx).await;
    if let Ok(Some(client)) = client {
        match body.action {
            RequestAction::Subscribe => {
                match Subscribers::add_subscriber(body.topic.clone(), client, subscriptions_tx).await {
                    Ok(result) => {
                        if result {
                            debug!("Subscribing to topic {}", body.topic.clone());
                            return Ok(StatusCode::OK)
                        } else {
                            return Err(warp::reject::reject())
                        }

                    }
                    Err(_) => Err(warp::reject::reject())
                }
            },
            RequestAction::Unsubscribe => {
                match Subscribers::remove_subscriber(body.topic.clone(), client, subscriptions_tx).await {
                    Ok(_) => {
                        debug!("Unsubscribing to topic {}", body.topic.clone());
                        Ok(StatusCode::OK)
                    },
                    Err(_) => Err(warp::reject::reject())
                }
            },
            _ => {
                error!("Error: unsubscribe_handler must be called with a request of unsubscribe");
                Err(warp::reject::reject())
            }
        }
    } else {
        Err(warp::reject::reject())
    }
}

#[cfg(test)]
mod tests {
    use warp::hyper::StatusCode;
    use crate::command::Command;
    use crate::store::{Client, Clients};
    use tokio::sync::mpsc;
    use crate::Reply;
    use std::collections::HashMap;
    use tokio::sync::Mutex;
    use std::sync::Arc;

    use super::register_handler;
    use super::unregister_handler;
    use super::health_handler;

    #[tokio::test]
    async fn test_register_handler() {
        let (clients_tx, mut clients_rx) = mpsc::channel::<Command<Client>>(32);
        let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

        tokio::spawn(async move {
            while let Some(cmd) = clients_rx.recv().await {
                match cmd {
                    Command::SetItem { key, value, responder } => {
                        let result = clients.lock().await.insert(key, value);
                        let _ = responder.send(result);
                    },
                    _ => panic!()
                }
            }
        });

        let result = register_handler(clients_tx).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().into_response().status(), 200);
    }

    #[tokio::test]
    async fn test_unregister_handler() {
        let (clients_tx, mut clients_rx) = mpsc::channel::<Command<Client>>(32);
        let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
        let client = Client {
            user_id: String::from("1"),
            sender: None
        };
        clients.lock().await.insert("1".to_string(), client);

        tokio::spawn(async move {
            while let Some(cmd) = clients_rx.recv().await {
                match cmd {
                    Command::UnsetItem { key, responder } => {
                        let result = clients.lock().await.remove(&key);
                        let _ = responder.send(result);
                    },
                    _ => panic!()
                }
            }
        });

        let result = unregister_handler("1".to_string(), clients_tx).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().into_response().status(), 200);
    }


    #[tokio::test]
    async fn test_health_handler() {
        let result = health_handler().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::OK);
    }
}