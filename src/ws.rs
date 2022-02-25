use std::{collections::HashSet};
use warp::ws::{Message, WebSocket};
use crate::{store::{Client, Command}, handler::{subscribe_handler, ping_handler, unsubscribe_handler, publish_handler}, serialize::{RequestAction, SocketRequest}};
use tokio::sync::mpsc::{self, Sender};
use futures::{StreamExt};
use tokio_stream::wrappers::UnboundedReceiverStream;
use serde_json::from_str;


pub async fn client_connection(ws: WebSocket, id: String, mut client: Client, subscriptions_tx: Sender<Command<HashSet<Client>>>, clients_tx: Sender<Command<Client>>, store_tx: Sender<Command<String>>) {
    let (client_ws_tx, mut client_ws_rx) = ws.split();
    let (client_tx, client_rx) = mpsc::unbounded_channel::<Result<Message, warp::Error>>();
    let client_rx = UnboundedReceiverStream::new(client_rx); 

    tokio::task::spawn(client_rx.forward(client_ws_tx));
    client.sender = Some(client_tx);
    Client::set_client(client, clients_tx.clone());

    println!("{} connected", id);

    while let Some(result) = client_ws_rx.next().await {
        let message = match result {
            Ok(message) => message,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_message(&id, message,subscriptions_tx.clone(), clients_tx.clone(),  store_tx.clone()).await;
    }

    // clients.lock().await.remove(&id);
    println!("{} disconnected", id);
}

// TODO: ask for the store tx here
async fn client_message(user_id: &str, msg: Message, subscriptions_tx: Sender<Command<HashSet<Client>>>, clients_tx: Sender<Command<Client>>, store_tx: Sender<Command<String>>) {
    println!("received message from {}: {:?}", user_id, msg);

    if msg.is_ping() {
        ping_handler(user_id, clients_tx.clone());
    }

    let message = match msg.to_str() {
        Ok(string) => string,
        Err(_) => {
            eprintln!("Error while parsing message to string");
            return;
        }
    };

    let socket_request: SocketRequest = match from_str(&message) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("Error while parsing socket request: {}", e);
            return;
        }
    };

    match socket_request.action {
        RequestAction::Subscribe => {
            subscribe_handler(socket_request, String::from(user_id), subscriptions_tx, clients_tx).await;
        },
        RequestAction::Unsubscribe => {
            unsubscribe_handler(socket_request, String::from(user_id), subscriptions_tx, clients_tx).await;
        },
        RequestAction::Set => {
            publish_handler(socket_request, String::from(user_id), subscriptions_tx, clients_tx, store_tx).await;
        },
        RequestAction::Remove => {
            publish_handler(socket_request, String::from(user_id),  subscriptions_tx, clients_tx, store_tx).await;
        }
    };
}