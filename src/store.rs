use std::{collections::{HashMap, HashSet}, hash::Hash, hash::Hasher, sync::{Arc}};
use tokio::{sync::{Mutex, mpsc::{self, Sender}, oneshot::{self, error::RecvError}}};
use warp::ws::Message;
use crate::command::{Command, get_value, set_value, remove_value, get_collection, add_value_to_collection, remove_value_from_collection};

pub type Responder<T> = oneshot::Sender<T>;

#[derive(Clone, Debug)]
pub struct Client {
    pub user_id: String,
    pub sender: Option<mpsc::UnboundedSender<Result<Message, warp::Error>>>
}

impl PartialEq for Client {
    fn eq(&self, other: &Client) -> bool {
        self.user_id == other.user_id
    }

    fn ne(&self, other: &Client) -> bool {
        self.user_id != other.user_id
    }
}

impl Eq for Client {}

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.user_id.hash(state);
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
pub type Clients = Arc<Mutex<HashMap<String, Client>>>;

pub struct Store;
impl Store {
    pub async fn set(key: String, value: String, store_tx: Sender<Command<String>>) -> Result<Option<String>, RecvError> {
        set_value(key, value, store_tx).await
    }

    pub async fn unset(key: String, store_tx: Sender<Command<String>>) -> Result<Option<String>, RecvError> {
        remove_value(key, store_tx).await
    }

    pub async fn add_to_collection(key: String, value: String, store_tx: Sender<Command<String>>) -> Result<bool, RecvError> {
        add_value_to_collection(key, value, store_tx).await
    }

    pub async fn remove_value_from_collection(key: String, value: String, store_tx: Sender<Command<String>>) -> Result<bool, RecvError> {
        remove_value_from_collection(key, value, store_tx).await
    }
}

pub struct Subscribers;
impl Subscribers {
    pub async fn get_subscribers(topic: String, subscriptions_tx: Sender<Command<Client>>) -> Result<Option<HashSet<Client>>, RecvError> {
        get_collection(topic, subscriptions_tx).await
    }

    pub async fn add_subscriber(topic: String, subscriber: Client, subscriptions_tx: Sender<Command<Client>>) -> Result<bool, RecvError> {
        // TODO: Add the ability to add a client to an existing set of clients who are subscribed to this topic
        add_value_to_collection(topic, subscriber, subscriptions_tx).await
    }

    pub async fn remove_subscriber(topic: String, subscriber: Client, subscriptions_tx: Sender<Command<Client>>) -> Result<bool, RecvError> {
        // TODO: Add the ability to remove a client from an existing set of clients who are subscribed to this topic
        remove_value_from_collection(topic, subscriber, subscriptions_tx).await
    }
}

pub type Subscriptions = Arc<Mutex<HashMap<String, HashSet<Client>>>>;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }

}