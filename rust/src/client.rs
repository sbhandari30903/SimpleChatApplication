use tokio::{
    net::TcpStream,
    io::{AsyncReadExt, AsyncWriteExt},
};
use std::io::{self, Write};
pub mod common;
use crate::common::ChatMessage;

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
        
        loop {
            match reader.read(&mut bytes).await {
                Ok(0) => break,
                Ok(n) => {
                    if let Ok(msg) = String::from_utf8(bytes[..n].to_vec()) {
                        if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&msg) {
                            println!("[{}] {}: {}", 
                                chat_msg.timestamp.format("%H:%M:%S"),
                                chat_msg.user_id,
                                chat_msg.content.trim()
                            );
                        } else {
                            // Handle system messages
                            if msg.contains("joined the chat") {
                                println!("{}", msg.trim().split(":").last().unwrap().trim_end_matches('}'));
                            } else if msg.contains("left the chat") {
                                println!("{}", msg.trim().split(":").last().unwrap().trim_end_matches('}'));
                            } else {
                                println!("{}", msg.trim());
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
        println!("Disconnected from server");
    });

    println!("Connected to chat server. Type your messages:");

    loop {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        
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