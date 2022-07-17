use crate::store::Responder;
use std::collections::HashSet;
use tokio::sync::{mpsc::Sender, oneshot::{self, error::RecvError}};
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
        Ok(result) => info!("#remove_value success: {:?}", result),
        Err(err) => error!("#remove_value error: {}", err)
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
        Ok(result) => info!("#remove_value success: {:?}", result),
        Err(err) => error!("#remove_value error: {}", err)
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
        Ok(result) => info!("#remove_value success: {:?}", result),
        Err(err) => error!("#remove_value error: {}", err)
    }
    
    resp_rx.await
}