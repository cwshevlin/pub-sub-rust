use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize, Deserializer, de::DeserializeOwned};
use tokio::sync::{Mutex, mpsc};
use warp::ws::Message;

#[derive(Deserialize, Serialize, Clone)]
pub struct Client {
    pub connection_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>
}

pub type Clients = Arc<Mutex<HashMap<String, Client>>>;
pub type Topics = Arc<Mutex<HashMap<String, Client>>>;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub user_id: usize,
}
  
#[derive(Serialize)]
pub struct RegisterResponse {
    pub url: String,
}
  
#[derive(Serialize, Deserialize)]
pub struct Event {
    pub topic: String,
    pub user_id: Option<usize>,
    pub message: String,
}

#[derive(Deserialize)]
pub struct TopicsRequest {
    topics: Vec<String>,
}
