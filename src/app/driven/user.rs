use crate::core::{
    models::User,
    ports::user as port,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct UserStore {
    users: Vec<User>,
}

pub struct UserMemRepo {
    store: Arc<Mutex<UserStore>>,
}

impl port::UserRepository for UserMemRepo {
    async fn add_user(&self, user: port::CreateUser) {
        let mut store = self.store.lock().await;
        store.users.push(User::new(user.id, user.username));
    }

    async fn get_user(&self, id: u32) -> Result<User, String> {
        let store = self.store.lock().await;

        let Some(user) = store.users.iter().find(|&user| user.id() == id) else {
            return Err("User not found".to_string());
        };

        Ok(user.clone())
    }

    async fn update_user(&self, user: port::UpdateUser) -> Result<(), String> {
        let mut store = self.store.lock().await;
        
        if let Some(existing_user) = store.users.iter_mut().find(|u| u.id() == user.id) {
            existing_user.with_username(user.username);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    async fn delete_user(&self, id: u32) -> Result<(), String> {
        let mut store = self.store.lock().await;
        if let Some(pos) = store.users.iter().position(|u| u.id() == id) {
            store.users.remove(pos);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
}

pub fn create_user_repo() -> UserMemRepo {
    UserMemRepo {
        store: Arc::new(Mutex::new(UserStore { users: Vec::new() })),
    }
}
