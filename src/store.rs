use std::{collections::{HashMap, HashSet}, hash::Hash, hash::Hasher, io::Error, sync::Arc};
use bytes::Bytes;
use tokio::sync::{Mutex, mpsc::{self, Sender}, oneshot};
use warp::ws::Message;
/*

Structure:

shard/
    chats/
        id/
            msg_read/
                id/
            msg/
                id/
    element_lists/
        id/
            elements/
                id/
                    data/
                    metadata/
            t: 123456789 (ignore)

This suggests the following top level tables:
    chats/<id>/msg
        id: "<serialized json>"
    chats/<id>/msg_read
        id: "<serialized json>"
    element_lists/<id>/elements
        id: "<serialized json>"

This is similar to how the manipulative children is represented in SQL

*/


#[derive(Clone, Debug)]
pub struct Client {
    pub user_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>
}

impl PartialEq for Client {
    fn eq(&self, other: &Client) -> bool {
        self.user_id == other.user_id
    }
}

impl Eq for Client {}

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.user_id.hash(state);
    }
}

pub type Clients = Arc<Mutex<HashMap<String, Client>>>;
pub type Subscribers = Arc<Mutex<HashMap<String, HashSet<Client>>>>;
type Responder<T> = oneshot::Sender<T>;

pub enum StoreCommand {
    Get {
        key: String,
        responder: Responder<Option<String>>,
    },
    Set {
        key: String,
        value: String,
        responder: Responder<()>,
    }
}
pub enum ClientsCommand {
    Get {
        key: String,
        responder: Responder<Option<Client>>
    },
    Insert {
        key: String,
        client: Client,
        responder: Responder<Option<Client>>
    },
    Remove {
        key: String,
        responder: Responder<Option<Client>>
    }
}

pub enum SubscribersCommand {
    Get {
        key: String,
        responder: Responder<Option<HashSet<Client>>>,
    },
    AddSubscriber {
        topic: String,
        subscriber: Client,
        responder: Responder<()>,
    }
}

pub async fn get_client(user_id: String, clients_tx: Sender<ClientsCommand>) -> Option<Client> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command = ClientsCommand::Get {
        key: user_id.clone(),
        responder: resp_tx
    };
    clients_tx.send(command).await;

    let result = resp_rx.await;

    match result {
        Ok(client) => client,
        Err(e) => None
    }
}

pub async fn insert_client(client: Client, clients_tx: Sender<ClientsCommand>) -> Option<Client> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command = ClientsCommand::Insert {
        key: client.user_id.clone(),
        client: client,
        responder: resp_tx
    };
    clients_tx.send(command).await;

    let result = resp_rx.await;

    match result {
        Ok(client) => client,
        Err(e) => None
    }
}

pub async fn remove_client(user_id: String, clients_tx: Sender<ClientsCommand>) -> Option<Client> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command = ClientsCommand::Remove {
        key: user_id,
        responder: resp_tx
    };
    clients_tx.send(command).await;

    let result = resp_rx.await;

    match result {
        Ok(client) => client,
        Err(e) => None
    }
}
