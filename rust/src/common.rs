use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub user_id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl ChatMessage {
    pub fn new(user_id: String, content: String) -> Self {
        Self {
            user_id,
            content,
            timestamp: Utc::now(),
        }
    }
}
