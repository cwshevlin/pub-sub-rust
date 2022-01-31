use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;
use client::Topics;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::net::{TcpListener, TcpStream};
use warp::{Rejection, Reply};
use warp::ws::WebSocket;
mod frame;
mod client;
use futures::StreamExt;
use futures::FutureExt;
use warp::Stream;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Multiple different commands are multiplexed over a single channel.
// #[derive(Debug)]

#[tokio::main]
async fn main() {
    let clients: client::Topics = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    // TODO CWS: this should be read as a TCP stream
    socket.readable().await;

    // Creating the buffer **after** the `await` prevents it from
    // being stored in the async task.
    let mut buf = [0; 128];

    // Try to read data, this may still fail with `WouldBlock`
    // if the readiness event is a false positive.
    match socket.try_read(&mut buf) {
        Ok(0) => (),
        Ok(n) => {
            respond(socket, &buf).await;
        }
        Err(e) => {
        }
    }
}

async fn respond(socket: TcpStream, &buf: &[u8; 128]) {
    // Wait for the socket to be writable
    socket.writable().await;

    // Try to write data, TODO: this may still fail with `WouldBlock`
    // if the readiness event is a false positive.
    match socket.try_write(&buf) {
        Ok(n) => (),
        Err(e) => ()
    }
}

pub async fn ws_handler(ws: warp::ws::Ws, id: String, topics: Topics) -> Result<impl Reply, Rejection> {
    let client = clients.lock().await.get(&id).cloned();
    match client {
      Some(c) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients, c))),
      None => Err(warp::reject::not_found()),
    }
}

pub async fn client_connection(ws: warp::ws::WebSocket, id: String, clients: Clients, mut client: client::Client) {
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