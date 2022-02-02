use warp::ws::WebSocket;
use crate::client::{Clients, Client};
use tokio::sync::mpsc;
use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;


pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_tx, client_rx) = mpsc::unbounded_channel();

    tokio::task::spawn(client_rx.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
        eprintln!("error sending websocket msg: {}", e);
        }
    }));
}