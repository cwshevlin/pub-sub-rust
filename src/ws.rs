use warp::ws::WebSocket;
use crate::client::{Clients, Client};
use tokio::sync::mpsc;
use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;


pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
    let (client_ws_tx, mut client_ws_rx) = ws.split();
    let (client_tx, client_rx) = mpsc::unbounded_channel::<Result<_, _>>();
    let client_rx = UnboundedReceiverStream::new(client_rx); 

    tokio::task::spawn(client_rx.forward(client_ws_tx).map(|result| {
        if let Err(e) = result {
            println!("error");
        }
        result
    }));
}