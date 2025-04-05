use tokio::{
    net::TcpStream,
    io::{AsyncReadExt, AsyncWriteExt},
};
use std::io::{self, Write};
use chrono::{Local};
use serde::{Serialize, Deserialize};
pub mod common;
use crate::common::ChatMessage;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
enum ServerMessage {
    UserJoined(String),
    UserLeft(String),
    ChatMessage(ChatMessage),
}

#[tokio::main]
async fn main() {
    // Ask for username before connecting
    print!("Enter your username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim().to_string();

    if username.is_empty() {
        println!("Username cannot be empty!");
        return;
    }

    let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let (mut reader, mut writer) = stream.into_split();

    // Send username first
    if let Err(_) = writer.write_all(username.as_bytes()).await {
        println!("Failed to send username");
        return;
    }
    writer.write_all(b"\n").await.unwrap();

    // Handle incoming messages
    tokio::spawn(async move {
        let mut bytes = vec![0; 1024];
        let mut buffer = String::new();
        
        loop {
            match reader.read(&mut bytes).await {
                Ok(0) => break,
                Ok(n) => {
                    if let Ok(chunk) = String::from_utf8(bytes[..n].to_vec()) {
                        buffer.push_str(&chunk);
                        
                        // Process complete messages
                        while let Some(pos) = buffer.find('\n') {
                            let msg_str = buffer[..pos].to_string();
                            buffer = buffer[pos + 1..].to_string();

                            if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&msg_str) {
                                match server_msg {
                                    ServerMessage::ChatMessage(chat_msg) => {
                                        let local_time = chat_msg.timestamp.with_timezone(&Local);
                                        println!("│ \x1b[90m[{}]\x1b[0m \x1b[32m{}\x1b[0m: {}", 
                                            local_time.format("%Y-%m-%d %H:%M:%S"),
                                            chat_msg.user_id,
                                            chat_msg.content.trim()
                                        );
                                    },
                                    ServerMessage::UserJoined(name) => {
                                        let local_time = Local::now();
                                        println!("└─ \x1b[90m[{}]\x1b[0m \x1b[33m{} joined the chat\x1b[0m", 
                                            local_time.format("%Y-%m-%d %H:%M:%S"),
                                            name
                                        );
                                    },
                                    ServerMessage::UserLeft(name) => {
                                        let local_time = Local::now();
                                        println!("└─ \x1b[90m[{}]\x1b[0m \x1b[31m{} left the chat\x1b[0m", 
                                            local_time.format("%Y-%m-%d %H:%M:%S"),
                                            name
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
        println!("\x1b[31m>>> Disconnected from server <<<\x1b[0m");
    });

    println!("\x1b[32m>>> Connected to chat server.\x1b[0m");

    loop {
        print!("\x1b[36m>\x1b[0m ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        if input.trim().is_empty() {
            continue;
        }

        if let Err(_) = writer.write_all(input.trim().as_bytes()).await {
            println!("Failed to send message");
            break;
        }
    }
}