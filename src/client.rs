use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot};
use warp::ws::Message;

pub struct Client {
    connection_id: usize,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>
}

pub type Topics = Arc<Mutex<HashMap<String, Client>>>;

pub fn register_client(connection_id: usize) {

}