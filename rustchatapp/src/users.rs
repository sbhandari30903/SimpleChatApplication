use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub id: u32,
    pub first_name: String,
    pub last_name: String,
}

pub struct UserStore {
    pub users: HashMap<u32, User>,
    pub next_id: u32,
}

impl UserStore {
    pub fn new() -> Self {
        UserStore {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_user(&mut self, first_name: &str, last_name: &str) -> User {
        let user = User {
            id: self.next_id,
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
        };
        self.users.insert(self.next_id, user.clone());
        self.next_id += 1;
        user
    }

    pub fn get_user_by_id(&self, user_id: u32) -> Option<User> {
        self.users.get(&user_id).cloned()
    }

    pub fn get_all_users(&self) -> Vec<User> {
        self.users.values().cloned().collect()
    }

    pub fn login_user(&self, first_name: &str, last_name: &str) -> Option<User> {
        self.users.values().find(|user| {
            user.first_name == first_name && user.last_name == last_name
        }).cloned()
    }
}

pub type SharedUserStore = Arc<Mutex<UserStore>>;
