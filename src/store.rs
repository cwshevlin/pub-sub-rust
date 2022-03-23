use std::{collections::{HashMap, HashSet}, hash::Hash, hash::Hasher, sync::{Arc}};
use tokio::{sync::{Mutex, mpsc::{self, Sender}, oneshot::{self, error::RecvError}}};
use warp::ws::Message;
use log::{info, error};
use settimeout::set_timeout;
use std::time::Duration;


#[derive(Clone, Debug)]
pub struct Client {
    pub user_id: String,
    pub sender: Option<mpsc::UnboundedSender<Result<Message, warp::Error>>>
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
    },
    AddToCollection {
        key: String,
        value: String,
        responder: Responder<Option<T>>,
    },
    RemoveFromCollection {
        key: String,
        value: String,
        responder: Responder<Option<T>>,
    }
}

pub async fn get_value<T>(key: String, sender: Sender<Command<T>>) -> Result<Option<T>, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::Get {
        key: key,
        responder: resp_tx
    };
    // TODO: better error handling here
    match sender.send(command).await {
        Ok(result) => info!("#get_value success: {:?}", result),
        Err(err) => error!("#get_value error: {}", err)
    }
    
    resp_rx.await
}

pub async fn set_value<T>(key: String, value: T, sender: Sender<Command<T>>) -> Result<Option<T>, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::Set {
        key: key,
        value: value,
        responder: resp_tx
    };
    match sender.send(command).await {
        Ok(result) => info!("#set_value success: {:?}", result),
        Err(err) => error!("#set_value error: {}", err)
    }
    
    resp_rx.await
}

pub async fn remove_value<T>(key: String, sender: Sender<Command<T>>) -> Result<Option<T>, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::Remove {
        key: key,
        responder: resp_tx
    };
    match sender.send(command).await {
        Ok(result) => info!("#remove_value success: {:?}", result),
        Err(err) => error!("#remove_value error: {}", err)
    }
    
    resp_rx.await
}

pub async fn add_value_to_collection<T>(key: String, value: String, sender: Sender<Command<T>>) -> Result<Option<T>, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::AddToCollection {
        key: key,
        value: value,
        responder: resp_tx
    };
    match sender.send(command).await {
        Ok(result) => info!("#remove_value success: {:?}", result),
        Err(err) => error!("#remove_value error: {}", err)
    }
    
    resp_rx.await
}

pub async fn remove_value_from_collection<T>(key: String, value: String, sender: Sender<Command<T>>) -> Result<Option<T>, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::RemoveFromCollection {
        key: key,
        value: value,
        responder: resp_tx
    };
    match sender.send(command).await {
        Ok(result) => info!("#remove_value success: {:?}", result),
        Err(err) => error!("#remove_value error: {}", err)
    }
    
    resp_rx.await
}

impl Store {
    pub async fn _get(key: String, store_tx: Sender<Command<String>>) -> Result<Option<String>, RecvError> {
        get_value(key, store_tx).await
    }

    pub async fn set(key: String, value: String, store_tx: Sender<Command<String>>) -> Result<Option<String>, RecvError> {
        set_value(key, value, store_tx).await
    }

    pub async fn remove(key: String, store_tx: Sender<Command<String>>) -> Result<Option<String>, RecvError> {
        remove_value(key, store_tx).await
    }

    pub async fn add_to_collection(key: String, value: String, store_tx: Sender<Command<String>>) -> Result<Option<String>, RecvError> {
        add_value_to_collection(key, value, store_tx).await
    }

    pub async fn remove_value_from_collection(key: String, value: String, store_tx: Sender<Command<String>>) -> Result<Option<String>, RecvError> {
        remove_value_from_collection(key, value, store_tx).await
    }
}

impl Client {
    pub async fn get_client(user_id: String, clients_tx: Sender<Command<Client>>) -> Result<Option<Client>, RecvError> {
        get_value(user_id, clients_tx).await
    }

    pub async fn set_client(client: Client, clients_tx: Sender<Command<Client>>) -> Result<Option<Client>, RecvError> {
        set_value(client.user_id.clone(), client, clients_tx).await
    }

    pub async fn remove_client(user_id: String, clients_tx: Sender<Command<Client>>) -> Result<Option<Client>, RecvError> {
        remove_value(user_id, clients_tx).await
    }
}

impl Subscribers {
    pub async fn get_subscribers(topic: String, subscriptions_tx: Sender<Command<HashSet<Client>>>) -> Result<Option<HashSet<Client>>, RecvError> {
        get_value(topic, subscriptions_tx).await
    }

    pub async fn add_subscriber(topic: String, subscriber: Client, subscriptions_tx: Sender<Command<HashSet<Client>>>) -> Result<Option<HashSet<Client>>, RecvError> {
        // TODO: Add the ability to add a client to an existing set of clients who are subscribed to this topic
        let client = subscriber.clone();
        tokio::spawn(async move {
            loop {
                set_timeout(Duration::from_secs(1)).await;
                client.sender.as_ref().unwrap().send(Ok(Message::text("{\"11\":\"112233\"}")));
            }
        });
        set_value(topic, HashSet::from([subscriber]), subscriptions_tx).await
    }

    pub async fn remove_subscriber(topic: String, subscriber: Client, subscriptions_tx: Sender<Command<HashSet<Client>>>) -> Result<Option<HashSet<Client>>, RecvError> {
        // TODO: Add the ability to remove a client from an existing set of clients who are subscribed to this topic
        set_value(topic, HashSet::from([subscriber]), subscriptions_tx).await
    }
}

