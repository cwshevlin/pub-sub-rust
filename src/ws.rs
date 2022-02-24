use std::{io::Error, collections::HashSet};

use warp::{ws::{Message, WebSocket}, Reply, Rejection, reply::Response};
use crate::{store::{Client, Command}, handler::{subscribe_handler, ping_handler, health_handler, unsubscribe_handler, publish_handler}, serialize::{RequestAction, SocketRequest}};
use tokio::sync::mpsc::{self, Sender};
use futures::{StreamExt};
use tokio_stream::wrappers::UnboundedReceiverStream;
use serde_json::from_str;


pub async fn client_connection(ws: WebSocket, id: String, mut client: Client, subscribers_tx: Sender<Command<HashSet<Client>>>, clients_tx: Sender<Command<Client>>) {
    let (client_ws_tx, mut client_ws_rx) = ws.split();
    let (client_tx, client_rx) = mpsc::unbounded_channel::<Result<Message, warp::Error>>();
    let client_rx = UnboundedReceiverStream::new(client_rx); 

    tokio::task::spawn(client_rx.forward(client_ws_tx));
    client.sender = Some(client_tx);
    Client::set_client(client, clients_tx.clone()).await;

    println!("{} connected", id);

    while let Some(result) = client_ws_rx.next().await {
        let message = match result {
            Ok(message) => message,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_message(&id, message, clients_tx.clone(), subscribers_tx.clone()).await;
    }

    // clients.lock().await.remove(&id);
    println!("{} disconnected", id);
}

// TODO: ask for the store tx here
async fn client_message(user_id: &str, msg: Message, clients_tx: Sender<Command<Client>>, subscribers_tx: Sender<Command<HashSet<Client>>>) {
    println!("received message from {}: {:?}", user_id, msg);

    if msg.is_ping() {
        ping_handler(user_id, clients_tx);
    }

    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    let socket_request: SocketRequest = match from_str(&message) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error while parsing message to subscribe request: {}", e);
            return;
        }
    };

    match socket_request.action {
        RequestAction::Subscribe => subscribe_handler(socket_request, String::from(user_id), subscribers_tx, clients_tx).await,
        RequestAction::Unsubscribe => unsubscribe_handler(socket_request, Store::from(user_id), subscribers_tx, clients_tx).await,
        RequestAction::Set => publish_handler(socket_request, user_id, subscribers_tx, clients_tx).await,
        RequestAction::Remove => publish_handler(socket_request, user_id,  subscribers_tx, clients_tx).await
    };
}