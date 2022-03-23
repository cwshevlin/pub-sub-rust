use serde::{Serialize, Deserialize};


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
    RemoveFromCollection,
    AddToCollection
}

#[derive(Deserialize, Debug)]
pub struct SocketRequest {
    pub action: RequestAction, 
    pub user_id: String,
    pub topic: String,
    pub message: Option<String>
}
