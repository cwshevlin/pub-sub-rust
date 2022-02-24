use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::{Mutex, mpsc};
use warp::ws::Message;
use serde_json::from_str;


#[derive(Deserialize)]
pub struct RegisterRequest {
    pub user_id: String,
}
  
#[derive(Serialize)]
pub struct RegisterResponse {
    pub url: String,
}
  
#[derive(Deserialize, Debug)]
pub enum RequestAction {
    Subscribe,
    Unsubscribe,
    Set,
    Remove
}

#[derive(Deserialize, Debug)]
pub struct SocketRequest {
    pub action: RequestAction, 
    pub user_id: String,
    pub topic: String,
    pub message: Option<String>
}
