use std::{collections::HashSet};
use warp::ws::{Message, WebSocket};
use crate::{store::{Client, Command}, handler::{subscribe_handler, ping_handler, unsubscribe_handler, publish_handler}, serialize::{RequestAction, SocketRequest}};
use tokio::sync::mpsc::{self, Sender};
use futures::{StreamExt};
use tokio_stream::wrappers::UnboundedReceiverStream;
use serde_json::from_str;
use log::{info, error};


pub async fn client_connection(ws: WebSocket, id: String, mut client: Client, subscriptions_tx: Sender<Command<Client>>, clients_tx: Sender<Command<Client>>, store_tx: Sender<Command<String>>) {
    let (client_ws_tx, mut client_ws_rx) = ws.split();
    let (client_tx, client_rx) = mpsc::unbounded_channel::<Result<Message, warp::Error>>();
    let client_rx = UnboundedReceiverStream::new(client_rx); 

    tokio::task::spawn(client_rx.forward(client_ws_tx));
    client.sender = Some(client_tx);
    match Client::set_client(client, clients_tx.clone()).await {
        Ok(result) => info!("set client result: {:?}", result),
        Err(err) => error!("set client error: {:?}", err)
    }

    while let Some(result) = client_ws_rx.next().await {
        let message = match result {
            Ok(message) => message,
            Err(err) => {
                error!("error receiving ws message for id: {}): {}", id.clone(), err);
                break;
            }
        };
        client_message(&id, message,subscriptions_tx.clone(), clients_tx.clone(),  store_tx.clone()).await;
    }

    match Client::remove_client(id, clients_tx.clone()).await {
        Ok(result) => info!("Client disconnected: {:?}", result.unwrap()),
        Err(_) => error!("get value error")
    }
}

async fn client_message(user_id: &str, msg: Message, subscriptions_tx: Sender<Command<Client>>, clients_tx: Sender<Command<Client>>, store_tx: Sender<Command<String>>) {
    if msg.is_ping() {
        match ping_handler(user_id, clients_tx.clone()).await {
            Ok(_) => info!("Ping from client {}", user_id),
            Err(err) => error!("Ping error: {:?}", err)
        }
        return;
    }

    println!("socket message: {:?}", msg.to_str());

    let message = match msg.to_str() {
        Ok(string) => string,
        Err(_) => {
            error!("Error while parsing message to string");
            return;
        }
    };

    let socket_request: SocketRequest = match from_str(&message) {
        Ok(request) => request,
        Err(err) => {
            error!("Error while parsing socket request: {}", err);
            return;
        }
    };

    match socket_request.action {
        RequestAction::Subscribe => {
            match subscribe_handler(socket_request, String::from(user_id), subscriptions_tx, clients_tx).await {
                Ok(_) => info!("client {} subscribed successfully", user_id),
                Err(_) => error!("#subscribe_handler error")
            }
        },
        RequestAction::Unsubscribe => {
            match unsubscribe_handler(socket_request, String::from(user_id), subscriptions_tx, clients_tx).await {
                Ok(_) => info!("client {} unsubscribed successfully", user_id),
                Err(_) => error!("#unsubscribe_handler error")
            }
        },
        RequestAction::Set => {
            match publish_handler(socket_request, String::from(user_id), subscriptions_tx, store_tx).await {
                Ok(_) => info!("client {} published successfully", user_id),
                Err(_) => error!("#publish_handler error")
            }
        },
        RequestAction::Unset => {
            match publish_handler(socket_request, String::from(user_id), subscriptions_tx, store_tx).await {
                Ok(_) => info!("client {} published successfully", user_id),
                Err(_) => error!("#publish_handler error")
            }
        },
        RequestAction::RemoveFromCollection => {
            match publish_handler(socket_request, String::from(user_id),  subscriptions_tx, store_tx).await {
                Ok(_) => info!("client {} removed value successfully", user_id),
                Err(_) => error!("#publish_handler error")

            }
        },
        RequestAction::AddToCollection => {
            match publish_handler(socket_request, String::from(user_id),  subscriptions_tx, store_tx).await {
                Ok(_) => info!("client {} added value successfully", user_id),
                Err(_) => error!("#publish_handler error")

            }
        }
    };
}