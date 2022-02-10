use crate::client::{Client, Clients, Event, RegisterRequest, SubscribeRequest, UnsubscribeRequest, Topics};
use uuid::Uuid;
use serde_json::{Value, json};
use warp::ws::Message;
use warp::{Rejection, hyper::StatusCode};
use crate::Reply;
use crate::ws;

pub async fn register_handler(body: RegisterRequest, clients: Clients) -> Result<impl Reply, Rejection> {
    let user_id = body.user_id;
    // let uuid = Uuid::new_v4().to_string();
    // TODO CWS: generate new uuids
    let uuid = String::from("cbf99b28-4488-45c9-aa12-46b3cfb979bb");
  
    match register_client(user_id.clone(), clients).await {
        Ok(_) => {
            return Ok(json!({
            "url": format!("ws://127.0.0.1:8000/ws/{}", user_id),
            }).to_string())
        },
        Err(_) => Err(warp::reject::reject())
    }
  }
    
async fn register_client(user_id: String, clients: Clients) -> Result<impl Reply, Rejection> {
    clients.lock().await.insert(
        user_id.clone(),
        Client {
            user_id: user_id.clone(),
            sender: None,
        },
    );
    Ok(StatusCode::OK)
}

pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply, Rejection> {
    clients.lock().await.remove(&id);
    Ok(StatusCode::OK)
}

pub async fn ws_handler(ws: warp::ws::Ws, id: String, topics: Topics, clients: Clients) -> Result<impl Reply, Rejection> {
    let client = clients.lock().await.get(&id).cloned();
    match client {
      Some(c) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, c, topics))),
      None => Err(warp::reject::not_found()),
    }
}

pub async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}

pub async fn publish_handler(body: Event, topics: Topics, clients: Clients) -> Result<impl Reply, Rejection> {
    let mut error = false;
    if let Some(clients) = topics.lock().await.get(&body.topic).cloned() {
        for client in clients {
            if let Some(sender) = &client.sender {
                match sender.send(Ok(Message::text(body.message.clone()))) {
                    Err(_) => error = true,
                    _ => {}
                }
            }
        }
        if !error {
            return Ok(StatusCode::OK);
        }
    }
    Ok(StatusCode::NOT_FOUND)
}

pub async fn subscribe_handler(body: SubscribeRequest,topics: Topics, clients: Clients) -> Result<impl Reply, Rejection> {
    println!("Client subscribing: {:#?}", clients);
    if let Some(client) = clients.lock().await.get(&body.user_id).cloned() {
        println!("Client subscribing: {:#?}", client);
        for topic in body.topics {
            if let Some(current_subscribers) = topics.lock().await.get(&topic) {
                let mut current_subscribers = current_subscribers.clone();
                current_subscribers.insert(client.clone());
                topics.lock().await.insert(
                    topic,
                    current_subscribers
                );
            }
        }
        if let Some(sender) = &client.sender {
            println!("Sending to client: {}", client.user_id);
            sender.send(Ok(Message::text(format!("subscribed"))));
        }
        return Ok(StatusCode::OK);
    }
    Ok(StatusCode::NOT_FOUND)
}

pub async fn unsubscribe_handler(body: UnsubscribeRequest, topics: Topics, clients: Clients) -> Result<impl Reply, Rejection> {
    if let Some(client) = clients.lock().await.get(&body.user_id).cloned() {
        for topic in body.topics {
            if let Some(current_subscribers) = topics.lock().await.get(&topic) {
                let mut current_subscribers = current_subscribers.clone();
                current_subscribers.remove(&client);
                topics.lock().await.insert(
                    topic,
                    current_subscribers
                );
            }
        }
        return Ok(StatusCode::OK);
    }
    Ok(StatusCode::NOT_FOUND)
}
