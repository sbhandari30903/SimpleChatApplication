use tokio::{
    net::TcpListener,
    sync::broadcast,
    io::{AsyncReadExt, AsyncWriteExt},
};
use serde::{Serialize, Deserialize};
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;
pub mod common;
use crate::common::ChatMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum ServerMessage {
    UserJoined(String),
    UserLeft(String),
    ChatMessage(ChatMessage),
}

struct ChatHistory {
    messages: Vec<ServerMessage>,
}

impl ChatHistory {
    fn new() -> Self {
        ChatHistory {
            messages: Vec::new(),
        }
    }

    fn add_message(&mut self, msg: ServerMessage) {
        self.messages.push(msg);
    }

    fn get_recent_messages(&self) -> Vec<ServerMessage> {
        self.messages.clone()
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let (tx, _) = broadcast::channel::<ServerMessage>(32);
    let history = Arc::new(Mutex::new(ChatHistory::new()));

    println!("Server running on 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let history = Arc::clone(&history);
        
        // Read username first
        let mut username = String::new();
        let mut username_bytes = vec![0; 1024];
        if let Ok(n) = socket.read(&mut username_bytes).await {
            username = String::from_utf8_lossy(&username_bytes[..n]).trim().to_string();
        }

        println!("New client connected: {} ({})", username, addr);
        
        // Send message history to new client
        let recent_messages = history.lock().await.get_recent_messages();
        for msg in recent_messages {
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.write_all((json + "\n").as_bytes()).await;
            }
        }

        // Announce new user to all existing clients
        let join_msg = ServerMessage::UserJoined(username.clone());
        history.lock().await.add_message(join_msg.clone());
        let _ = tx.send(join_msg);

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.into_split();
            let mut buf = vec![0; 1024];

            let write_username = username.clone();

            let write_task = tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    let json = match msg {
                        ServerMessage::UserJoined(name) => {
                            if name == write_username { continue; }
                            format!("{{\"type\":\"system\",\"content\":\"User {} joined the chat\"}}\n", name)
                        },
                        ServerMessage::UserLeft(name) => {
                            if name == write_username { continue; }
                            format!("{{\"type\":\"system\",\"content\":\"User {} left the chat\"}}\n", name)
                        },
                        ServerMessage::ChatMessage(chat_msg) => {
                            if chat_msg.user_id == write_username { continue; }
                            serde_json::to_string(&chat_msg).unwrap() + "\n"
                        }
                    };
                    
                    if writer.write_all(json.as_bytes()).await.is_err() {
                        break;
                    }
                }
            });

            // Read messages from client
            loop {
                let n = match reader.read(&mut buf).await {
                    Ok(0) => break, // Connection closed
                    Ok(n) => n,
                    Err(_) => break,
                };

                let message = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                let chat_msg = ChatMessage::new(username.clone(), message);
                let server_msg = ServerMessage::ChatMessage(chat_msg);
                
                history.lock().await.add_message(server_msg.clone());
                let _ = tx.send(server_msg);
            }

            // Send leave notification before cleaning up
            let leave_msg = ServerMessage::UserLeft(username.clone());
            history.lock().await.add_message(leave_msg.clone());
            let _ = tx.send(leave_msg);
            write_task.abort();
            println!("Client disconnected: {} ({})", username, addr);
        });
    }
}