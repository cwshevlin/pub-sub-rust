use crate::store::Responder;
use std::collections::HashSet;
use tokio::sync::{mpsc::Sender, oneshot::{self, error::RecvError}};


#[derive(Debug)]
pub enum Command<T> {
    // TODO: change String to &str?
    GetItem {
        key: String,
        responder: Responder<Option<T>>,
    },
    SetItem {
        key: String,
        value: T,
        responder: Responder<Option<T>>,
    },
    UnsetItem {
        key: String,
        responder: Responder<Option<T>>,
    },
    GetCollection {
        key: String,
        responder: Responder<Option<HashSet<T>>>,
    },
    AddToCollection {
        key: String,
        value: T,
        responder: Responder<bool>,
    },
    RemoveFromCollection {
        key: String,
        value: T,
        responder: Responder<bool>,
    }
}

pub async fn get_value<T>(key: String, sender: Sender<Command<T>>) -> Result<Option<T>, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::GetItem {
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
    let command =  Command::<T>::SetItem {
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
    let command =  Command::<T>::UnsetItem {
        key: key,
        responder: resp_tx
    };
    match sender.send(command).await {
        Ok(result) => info!("#remove_value success: {:?}", result),
        Err(err) => error!("#remove_value error: {}", err)
    }
    
    resp_rx.await
}

pub async fn add_value_to_collection<T>(key: String, value: T, sender: Sender<Command<T>>) -> Result<bool, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::AddToCollection {
        key: key,
        value: value,
        responder: resp_tx
    };
    match sender.send(command).await {
        Ok(result) => info!("#add_value_to_collection success: {:?}", result),
        Err(err) => error!("#add_value_to_collection error: {}", err)
    }
    
    resp_rx.await
}

pub async fn remove_value_from_collection<T>(key: String, value: T, sender: Sender<Command<T>>) -> Result<bool, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::RemoveFromCollection {
        key: key,
        value: value,
        responder: resp_tx
    };
    match sender.send(command).await {
        Ok(result) => info!("#remove_value_from_collection success: {:?}", result),
        Err(err) => error!("#remove_value_from_collection error: {}", err)
    }
    
    resp_rx.await
}

pub async fn get_collection<T>(key: String, sender: Sender<Command<T>>) -> Result<Option<HashSet<T>>, RecvError> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let command =  Command::<T>::GetCollection {
        key: key,
        responder: resp_tx
    };
    match sender.send(command).await {
        Ok(result) => info!("#get_collection success: {:?}", result),
        Err(err) => error!("#get_collection error: {}", err)
    }
    
    resp_rx.await
}

#[cfg(test)]
mod tests {

use super::*;
use tokio::sync::mpsc::channel;
use crate::store::{Client, Clients, Subscriptions};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::println as info;

    #[tokio::test]
    async fn test_get_value() {
        let key: String = String::from("hello");
        let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
        let client = Client {
            user_id: String::from("1"),
            sender: None
        };
        clients.lock().await.insert(key, client);
        let (clients_tx, mut clients_rx) = channel::<Command<Client>>(32);
        tokio::spawn(async move {
            while let Some(cmd) = clients_rx.recv().await {
                match cmd {
                    Command::GetItem { key, responder } => {
                        if let Some(result) = clients.lock().await.get(&key) {
                        let _ = responder.send(Some(result.clone()));
                        }
                    },
                    _ => {
                        error!("Only Get, Set and Unset may be used with clients.");
                    }
                }
            }
        });

        let result = get_value(String::from("hello"), clients_tx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_value() {
        let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
        let client = Client {
            user_id: String::from("1"),
            sender: None
        };
        let (clients_tx, mut clients_rx) = channel::<Command<Client>>(32);
        tokio::spawn(async move {
            while let Some(cmd) = clients_rx.recv().await {
                match cmd {
                    Command::SetItem { key, value, responder } => {
                        let result = clients.lock().await.insert(key, value);
                        let _ = responder.send(result);
                    },
                    _ => {
                        error!("Set may be used.");
                    }
                }
            }
        });

        let result = set_value(String::from("hello"), client, clients_tx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_value() {
        //Setup
        let key: String = String::from("hello");
        let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
        let client = Client {
            user_id: String::from("1"),
            sender: None
        };
        clients.lock().await.insert(key, client);
        let (clients_tx, mut clients_rx) = channel::<Command<Client>>(32);

        tokio::spawn(async move {
            while let Some(cmd) = clients_rx.recv().await {
                match cmd {
                    Command::UnsetItem { key, responder } => {
                        info!("Unset key: {:?} in the client store.", key);
                        let result = clients.lock().await.remove(&key);
                        let _ = responder.send(result);
                    },
                    _ => {
                        error!("Only Get and Unset may be used");
                    }
                }
            }
        });

        let result = remove_value(String::from("hello"), clients_tx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_collection() {
        let subscriptions: Subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let (subscriptions_tx, mut subscriptions_rx) = channel::<Command<Client>>(32);
        let key: String = String::from("hello");
        let client = Client {
            user_id: String::from("1"),
            sender: None
        };
        let mut clients = HashSet::new();
        clients.insert(client.clone());
        subscriptions.lock().await.insert(key, clients);
        tokio::spawn(async move {
            while let Some(cmd) = subscriptions_rx.recv().await {
                match cmd {
                    Command::GetCollection { key, responder } => {
                        if let Some(result) = subscriptions.lock().await.get(&key) {
                        info!("Get key {:?} from the subscriptions store. Result: {:?}", key, result);
                        let _ = responder.send(Some(result.clone()));
                        }
                    },
                    _ => {
                        error!("Only Get, Set and Unset may be used with subscriptions.");
                    }
                }
            }
        });
        let result = get_collection(String::from("hello"), subscriptions_tx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_value_to_collection() {
        let subscriptions: Subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let (subscriptions_tx, mut subscriptions_rx) = channel::<Command<Client>>(32);
        let client = Client {
            user_id: String::from("1"),
            sender: None
        };
        let key: String = String::from("hello");
        tokio::spawn(async move {
            while let Some(cmd) = subscriptions_rx.recv().await {
                match cmd {
                    Command::AddToCollection { key, value, responder } => {
                        let mut subscriptions = subscriptions.lock().await;
                        let subscriptions_option = subscriptions.get_mut(&key);
                        let result = match subscriptions_option {
                        Some(subscription) => subscription.insert(value),
                        None => {
                            let mut subscription = HashSet::new();
                            subscription.insert(value);
                            let insert_result = subscriptions.insert(key, subscription);
                            if insert_result.is_some() {
                                false
                            } else {
                                true
                            }
                        }
                        };
                        let _ = responder.send(result);
                    }
                    _ => {
                        error!("Only Get, Set and Unset may be used with subscriptions.");
                    }
                }
            }
        });
        let result = add_value_to_collection(key, client, subscriptions_tx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_value_from_collection() {
        let subscriptions: Subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let (subscriptions_tx, mut subscriptions_rx) = channel::<Command<Client>>(32);
        let client = Client {
            user_id: String::from("1"),
            sender: None
        };
        let mut clients = HashSet::new();
        clients.insert(client.clone());
        let key: String = String::from("hello");
        subscriptions.lock().await.insert(key, clients);

        tokio::spawn(async move {
            while let Some(cmd) = subscriptions_rx.recv().await {
            // TODO: pass the data structure here so that it is the only one that has access?
                match cmd {
                    Command::RemoveFromCollection { key, value, responder } => {
                        let mut subscriptions = subscriptions.lock().await;
                        let subscriptions_option = subscriptions.get_mut(&key);
                        let result = match subscriptions_option {
                            Some(collection) => collection.remove(&value),
                            None => false
                        };
                        info!("Remove key {:?} from the subscriptions store. Result: {:?}", key, result);
                        let _ = responder.send(result);
                    },
                    _ => {
                        error!("Only Get, Set and Unset may be used with subscriptions.");
                    }
                }
            }
        });

        let result = remove_value_from_collection(String::from("hello"), client, subscriptions_tx).await;
        assert!(result.is_ok());
    }

}
