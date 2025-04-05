use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, Responder, Error};
use actix_web_actors::ws;
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::messages::{save_message, get_messages, Message};
use crate::users::{SharedUserStore, UserStore, User};  // Import User and SharedUserStore
use actix_cors::Cors;
use log::info;
use env_logger;

mod messages;
mod users;

#[derive(Message)]
#[rtype(result = "()")]
struct WsMessage(String);

#[derive(Debug)]
struct UserWebSocket {
    user_id: u32,
    user_connections: Arc<Mutex<HashMap<u32, Addr<UserWebSocket>>>>,
}

impl UserWebSocket {
    fn new(
        user_id: u32,
        user_connections: Arc<Mutex<HashMap<u32, Addr<UserWebSocket>>>>,
    ) -> Self {
        UserWebSocket {
            user_id,
            user_connections,
        }
    }
}

impl Actor for UserWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut user_connections = self.user_connections.lock().unwrap();
        user_connections.insert(self.user_id, ctx.address());
        info!("User {} connected.", self.user_id); // Log when the user connects

    }

    fn stopped(&mut self, _: &mut Self::Context) {
        let mut user_connections = self.user_connections.lock().unwrap();
        user_connections.remove(&self.user_id);
    }
}

impl Handler<WsMessage> for UserWebSocket {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UserWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(text)) => {
                let message_data: Result<serde_json::Value, _> = serde_json::from_str(&text);

                if let Ok(data) = message_data {
                    if let (Some(sender_id), Some(receiver_id), Some(content)) = (
                        data.get("sender_id").and_then(|v| v.as_u64()).map(|v| v as u32),
                        data.get("receiver_id").and_then(|v| v.as_u64()).map(|v| v as u32),
                        data.get("content").and_then(|v| v.as_str()),
                    ) {

                        info!("Received message from user {} to user {}: {}", sender_id, receiver_id, content);

                        // Save the message in the in-memory store using messages.rs
                        save_message(sender_id, receiver_id, content.to_string());

                        // Broadcast the message to the receiver's WebSocket
                        self.broadcast_message(receiver_id, sender_id, content.to_string());
                    }
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl UserWebSocket {
    fn broadcast_message(&self, receiver_id: u32, sender_id: u32, content: String) {
        let user_connections = self.user_connections.lock().unwrap();
        
        if let Some(receiver_ws) = user_connections.get(&receiver_id) {
            // Logging receiver address
            info!("Broadcasting message to user {}: {}", receiver_id, content);
            
            // Create the message to be sent
            let message = Message {
                sender_id,
                receiver_id,
                content,
                timestamp: Utc::now(),
            };
            let message_str = serde_json::to_string(&message).unwrap();
            
            // Send the message to the receiver's WebSocket
            receiver_ws.do_send(WsMessage(message_str));
        } else {
            // Log if no active WebSocket connection is found for the receiver
            info!("No active connection found for receiver user {}", receiver_id);
        }
    }
}


// WebSocket connection route
async fn ws_connect(
    req: HttpRequest,
    stream: web::Payload,
    user_connections: web::Data<Arc<Mutex<HashMap<u32, Addr<UserWebSocket>>>>>,
) -> Result<impl Responder, Error> {

    let user_id: u32 = req
    .uri()
    .query()
    .and_then(|q| {
        url::form_urlencoded::parse(q.as_bytes())
            .find(|(key, _)| key == "userId")
            .and_then(|(_, value)| value.parse().ok())
    })
    .unwrap_or(0); // Default user_id to 0 if not found

info!("Received user_id: {}", user_id);


    let user_ws = UserWebSocket::new(
        user_id,
        user_connections.get_ref().clone(),
    );
    ws::start(user_ws, &req, stream)
}

// Declare routes
async fn home() -> impl Responder {
    HttpResponse::Ok().body("Hello from Home")
}

#[derive(Deserialize)]
struct RegisterUser {
    first_name: String,
    last_name: String,
}

async fn register(
    user_data: web::Json<RegisterUser>,
    store: web::Data<SharedUserStore>,
) -> impl Responder {
    let mut store = store.lock().unwrap();
    let new_user = store.add_user(&user_data.first_name, &user_data.last_name);

    HttpResponse::Ok().json(new_user)
}

async fn get_all_users(store: web::Data<SharedUserStore>) -> impl Responder {
    let store = store.lock().unwrap();
    
    // Convert the HashMap into a Vec of users (this already happens in your code)
    let users: Vec<User> = store.users.values().cloned().collect();
    
    // Return the users as a JSON array
    HttpResponse::Ok().json(users)
}

#[derive(Deserialize)]
struct LoginUser {
    first_name: String,
    last_name: String,
}

async fn login(
    user_data: web::Json<LoginUser>,
    store: web::Data<SharedUserStore>,
) -> impl Responder {
    let store = store.lock().unwrap();
    match store.login_user(&user_data.first_name, &user_data.last_name) {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().body("User not found"),
    }
}

// New route to get messages between two users
#[derive(Deserialize)]
struct MessageQuery {
    userId1: u32,
    userId2: u32,
}

async fn get_chat_history(
    query: web::Query<MessageQuery>,
) -> impl Responder {
    let messages = get_messages(query.userId1, query.userId2);

    HttpResponse::Ok().json(messages)
}

fn handle_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(home))
       .route("/register", web::post().to(register))
       .route("/login", web::post().to(login))
       .route("/users", web::get().to(get_all_users))
       .route("/ws", web::get().to(ws_connect))
       .route("/messages", web::get().to(get_chat_history));  // Add new route for messages

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let store = Arc::new(Mutex::new(UserStore::new()));
    let user_connections = Arc::new(Mutex::new(HashMap::<u32, Addr<UserWebSocket>>::new()));

    println!("Starting server at http://127.0.0.1:8082");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(store.clone()))
            .app_data(web::Data::new(user_connections.clone()))
            .wrap(Cors::permissive())  // This line enables permissive CORS
            .configure(handle_routes)
    })
    .bind("127.0.0.1:8082")?
    .run()
    .await
}
