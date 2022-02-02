use crate::client::{RegisterRequest, Client, Clients, Event};
use uuid::Uuid;
use serde_json::{Value, json};
use warp::ws::Message;
use warp::{Rejection, hyper::StatusCode};
use crate::Reply;
use crate::ws;

// TODO CWS: can all of these be a impl of Reply??

pub async fn register_handler(body: RegisterRequest, clients: Clients) -> Result<Value, Rejection> {
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().simple().to_string();
  
    register_client(uuid.clone(), user_id, clients).await;
    Ok(json!({
      "url": format!("ws://127.0.0.1:8000/ws/{}", uuid),
    }))
  }
    
async fn register_client(id: String, user_id: usize, clients: Clients) -> Result<Value, Rejection> {
    clients.lock().await.insert(
        id,
        Client {
            connection_id: user_id,
            topics: vec![String::from("cats")],
            sender: None,
        },
    );
    Ok(json!({}))
}

pub async fn unregister_handler(id: String, clients: Clients) -> Result<Value, Rejection> {
    clients.lock().await.remove(&id);
    Ok(json!({}))
}

pub async fn ws_handler(ws: warp::ws::Ws, id: String, clients: Clients) -> Result<impl Reply, Rejection> {
    let client = clients.lock().await.get(&id).cloned();
    match client {
      Some(c) => {
          Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, c)))
      },
      None => Err(warp::reject::not_found()),
    }
}

pub async fn health_handler() -> Result<Value, Rejection> {
    Ok(json!({}))
}

pub async fn publish_handler(body: Event, clients: Clients) -> Result<Value, Rejection> {
    clients
        .lock()
        .await
        .iter_mut()
        .filter(|(_, client)| match body.user_id {
        Some(v) => client.connection_id == v,
        None => true,
        })
        .filter(|(_, client)| client.topics.contains(&body.topic))
        .for_each(|(_, client)| {
        if let Some(sender) = &client.sender {
            let _ = sender.send(Ok(Message::text(body.message.clone())));
        }
    });

    Ok(json!({}))
}
