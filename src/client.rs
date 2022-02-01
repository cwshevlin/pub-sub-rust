use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use warp::ws::Message;

#[derive(Clone)]
pub struct Client {
    pub connection_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>
}

pub type Clients = Arc<Mutex<HashMap<String, Client>>>;
pub type Topics = Arc<Mutex<HashMap<String, Client>>>;
pub struct RegisterRequest {
    pub user_id: usize,
  }
  
pub struct RegisterResponse {
    pub url: String,
}
  
pub struct Event {
    pub topic: String,
    pub user_id: Option<usize>,
    pub message: String,
}

pub struct TopicsRequest {
    topics: Vec<String>,
}
