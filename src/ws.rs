use warp::ws::{Message, WebSocket};
use crate::client::{Client, Clients, TopicsRequest};
use tokio::sync::mpsc;
use futures::{StreamExt};
use tokio_stream::wrappers::UnboundedReceiverStream;
use serde::{Deserialize};
use serde_json::from_str;


pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
    let (client_ws_tx, mut client_ws_rx) = ws.split();
    let (client_tx, client_rx) = mpsc::unbounded_channel::<Result<Message, warp::Error>>();
    let client_rx = UnboundedReceiverStream::new(client_rx); 

    tokio::task::spawn(client_rx.forward(client_ws_tx));
    client.sender = Some(client_tx);
    clients.lock().await.insert(id.clone(), client);

    println!("{} connected", id);

    while let Some(result) = client_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_msg(&id, msg, &clients).await;
    }

    clients.lock().await.remove(&id);
    println!("{} disconnected", id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients) {
    println!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    if message == "ping" || message == "ping\n" {
        return;
    }

    // TODO CWS: parse subscribe/unsubscribe
    let topics_req: TopicsRequest = match from_str(&message) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error while parsing message to topics request: {}", e);
            return;
        }
    };

    // TODO CWS: parse publish messages, maybe add/delete/update
}