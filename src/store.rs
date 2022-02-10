use crate::client::{Client, Clients, Topics};
use warp::ws::Message;
/*

Structure:

shard/
    chats/
        id/
            msg_read/
                id/
            msg/
                id/
    element_lists/
        id/
            elements/
                id/
                    data/
                    metadata/
            t: 123456789 (ignore)

This suggests the following top level tables:
    chats/<id>/msg
        id: "<serialized json>"
    chats/<id>/msg_read
        id: "<serialized json>"
    element_lists/<id>/elements
        id: "<serialized json>"

This is similar to how the manipulative children is represented in SQL

*/
enum StoreCommand {
    Get = "HGET",
    Set = "HSET",
    Delete = "HDELETE"
}

struct Store {
    data: Arc<Mutex<HashMap<String, String>>>
}

impl Store {
    pub fn get() -> String {
        todo!();
    }

    pub fn delete() {
        todo!();
    }

    pub fn insert() {
        todo!();
    }
}

pub fn handle_update(command: StoreCommand, key: String, value: String, topics: Topics, clients: Clients) {
    match command {
        StoreCommand::Get => println!("get"),
        StoreCommand::Set => println!("set"),
        StoreCommand::Delete => println!("delete"),
    }
}

pub fn handle_set(key: String, value: String, topics: Topics, clients: Clients, data: Store) -> Result {
    data.lock().await.insert(key, value);
    let clients = topics.lock().await.get(key);
    if let Some(clients) = topics.lock().await.get(key).cloned() {
        for client in clients {
            if let Some(sender) = &client.sender {
                sender.send(Ok(Message::text("{{}: {}}", key, value)));
                return Ok()
            }
        }
    }
    Err
}

pub fn handle_delete(key: String, value: String, topics: Topics, clients: Clients, data: Store) -> Result {
    data.lock().await.remove(key);
    let clients = topics.lock().await.get(key);
    // TODO CWS: refactor this to share with the above
    if let Some(clients) = topics.lock().await.get(key).cloned() {
        for client in clients {
            if let Some(sender) = &client.sender {
                sender.send(Ok(Message::text("{} removed", key)));
                return Ok()
            }
        }
    }
    Err
}