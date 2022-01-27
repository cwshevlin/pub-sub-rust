use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};
use tokio::net::{TcpListener, TcpStream};
mod frame;
use frame::Frame;

/// Multiple different commands are multiplexed over a single channel.
// #[derive(Debug)]

/// Provided by the requester and used by the manager task to send the command
/// response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
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
    let mut buf = [0; 64];

    // Try to read data, this may still fail with `WouldBlock`
    // if the readiness event is a false positive.
    match socket.try_read(&mut buf) {
        Ok(0) => (),
        Ok(n) => {
            let frame = Frame::new(&buf);
            respond(socket, &buf).await;
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

async fn respond(socket: TcpStream, &buf: &[u8; 64]) {
    // Wait for the socket to be writable
    socket.writable().await;

    // Try to write data, TODO: this may still fail with `WouldBlock`
    // if the readiness event is a false positive.
    match socket.try_write(&buf) {
        Ok(n) => (),
        Err(e) => ()
    }
}