use std::{collections::{HashMap, HashSet}, hash::Hash, hash::Hasher, io::Error, sync::{Arc, mpsc::Receiver}};
use bytes::Bytes;
use tokio::sync::{Mutex, mpsc::{self, Sender}, oneshot::{self, error::RecvError}};
use warp::ws::Message;


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

pub struct Store;
pub struct Subscribers;

pub type Clients = Arc<Mutex<HashMap<String, Client>>>;
pub type Subscriptions = Arc<Mutex<HashMap<String, HashSet<Client>>>>;
type Responder<T> = oneshot::Sender<T>;

pub enum Command<T> {
    // TODO: change String to &str?
    Get {
        key: String,
        responder: Responder<Option<T>>,
    },
    Set {
        key: String,
        value: T,
        responder: Responder<Option<T>>,
    },
    Remove {
        key: String,
        responder: Responder<Option<T>>,
    }
}

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

pub async fn get_value<T>(key: String, sender: Sender<Command<T>>) -> Result<Option<T>, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::Get {
        key: key,
        responder: resp_tx
    };
    sender.send(command).await;
    
    resp_rx.await
}

pub async fn set_value<T>(key: String, value: T, sender: Sender<Command<T>>) -> Result<Option<T>, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::Set {
        key: key,
        value: value,
        responder: resp_tx
    };
    sender.send(command).await;
    
    resp_rx.await
}

impl Store {
    pub async fn get(key: String, store_tx: Sender<Command<String>>) -> Option<String> {
        let result = get_value(key, store_tx).await;

        match result {
            Ok(value) => value,
            Err(e) => None
        }
    }
}

impl Client {
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

    pub async fn insert_client(client: Client, clients_tx: Sender<Command<Client>>) -> Option<Client> {
        let result = set_value(client.user_id.clone(), client, clients_tx).await;
        match result {
            Ok(client) => client,
            Err(e) => Nonex
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
}

impl Subscribers {
    pub async fn get(topic: String, subscription_tx: Sender<SubscribersCommand>) -> Option<HashSet<Client>> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let command = SubscribersCommand::Get {
            key: topic,
            responder: resp_tx
        };

        subscription_tx.send(command).await;

        let result = resp_rx.await;

        match result {
            Ok(subscribers) => subscribers,
            Err(e) => None
        }
    }

    pub async fn add_subscriber(topic: String, subscriber: Client, subscription_tx: Sender<SubscribersCommand>) {

    }
}

