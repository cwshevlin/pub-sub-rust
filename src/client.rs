use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::{Mutex, mpsc};
use warp::ws::Message;
use std::hash::{Hash, Hasher};

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
pub type Topics = Arc<Mutex<HashMap<String, HashSet<Client>>>>;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub user_id: String,
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


#[derive(Deserialize, Debug)]
pub struct SubscribeRequest {
    pub user_id: String,
    pub topics: Vec<String>,
}

#[derive(Deserialize)]
pub struct UnsubscribeRequest {
    pub user_id: String,
    pub topics: Vec<String>,
}