use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use lazy_static::lazy_static;

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub sender_id: u32,
    pub receiver_id: u32,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

type MessageStore = Arc<Mutex<HashMap<String, Vec<Message>>>>;

lazy_static! {
    pub static ref MESSAGE_STORE: MessageStore = Arc::new(Mutex::new(HashMap::new()));
}

// Generate a consistent key for storing/retrieving messages
pub fn get_message_key(id1: u32, id2: u32) -> String {
    if id1 < id2 {
        format!("{}_{}", id1, id2)
    } else {
        format!("{}_{}", id2, id1)
    }
}

// Save a message to the in-memory store
pub fn save_message(sender_id: u32, receiver_id: u32, content: String) {
    let msg = Message {
        sender_id,
        receiver_id,
        content,
        timestamp: Utc::now(),
    };

    let key = get_message_key(sender_id, receiver_id);
    let mut store = MESSAGE_STORE.lock().unwrap();
    store.entry(key).or_default().push(msg);
}

// Retrieve the chat history between two users
pub fn get_messages(user_id1: u32, user_id2: u32) -> Vec<Message> {
    let key = get_message_key(user_id1, user_id2);
    
    let store = MESSAGE_STORE.lock().unwrap();
    store.get(&key).cloned().unwrap_or_default()
}
