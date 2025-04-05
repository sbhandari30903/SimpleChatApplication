use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::VecDeque;

const MAX_HISTORY_SIZE: usize = 100;  // Keep last 100 messages

#[derive(Debug)]
struct ChatHistory {
    messages: VecDeque<ServerMessage>,
}

impl ChatHistory {
    fn new() -> Self {
        Self {
            messages: VecDeque::with_capacity(MAX_HISTORY_SIZE),
        }
    }

    fn add_message(&mut self, message: ServerMessage) {
        if self.messages.len() >= MAX_HISTORY_SIZE {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }

    fn get_recent_messages(&self) -> Vec<ServerMessage> {
        self.messages.iter().cloned().collect()
    }
}