use tokio::{
    net::TcpListener,
    sync::broadcast,
    io::{AsyncReadExt, AsyncWriteExt},
};
use serde_json;
use rand::random;
pub mod common;
use crate::common::ChatMessage;

#[derive(Debug, Clone)]
enum ServerMessage {
    UserJoined(String),
    UserLeft(String),
    ChatMessage(ChatMessage),
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let (tx, _) = broadcast::channel::<ServerMessage>(32);

    println!("Server running on 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        
        // Read username first
        let mut username = String::new();
        let mut username_bytes = vec![0; 1024];
        if let Ok(n) = socket.read(&mut username_bytes).await {
            username = String::from_utf8_lossy(&username_bytes[..n]).trim().to_string();
        }

        println!("New client connected: {} ({})", username, addr);
        
        // Announce new user to all existing clients
        let _ = tx.send(ServerMessage::UserJoined(username.clone()));

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

            loop {
                let n = match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => break,
                };

                let message = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                let chat_msg = ChatMessage::new(username.clone(), message);
                
                let _ = tx.send(ServerMessage::ChatMessage(chat_msg));
            }

            write_task.abort();
            let _ = tx.send(ServerMessage::UserLeft(username.clone()));
            println!("Client disconnected: {} ({})", username, addr);
        });
    }
}