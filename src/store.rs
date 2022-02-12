use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::client::{Client, Clients, Topics};
use bytes::Bytes;
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

pub enum Command {
    Get {
        key: String,
    },
    Set {
        key: String,
        val: Bytes,
    }
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
