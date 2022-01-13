use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};
use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};

/// Multiple different commands are multiplexed over a single channel.
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

/// Provided by the requester and used by the manager task to send the command
/// response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

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
    // TODO CWS: this should be read as a TCP stream with a command and 
    socket.readable().await?;

    // Creating the buffer **after** the `await` prevents it from
    // being stored in the async task.
    let mut buf = [0; 4096];


    // Try to read data, this may still fail with `WouldBlock`
    // if the readiness event is a false positive.
    match stream.try_read(&mut buf) {
        Ok(0) => break,
        Ok(n) => {
            println!("read {} bytes", n);
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            continue;
        }
        Err(e) => {
            return Err(e.into());
        }
    }
}