use crate::core::{models::User, ports::user as port};
use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct UserStore {
    users: Vec<User>,
}

pub struct UserMemRepo {
    store: Arc<Mutex<UserStore>>,
}

#[async_trait]
impl port::UserRepository for UserMemRepo {
    async fn add_user(&self, create_user: port::CreateUser) -> User {
        let mut store = self.store.lock().await;
        let user = User::new(create_user.id, create_user.username);
        let user_clone = user.clone();
        store.users.push(user_clone);
        user
    }

    async fn get_user(&self, id: Uuid) -> Result<User, String> {
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

    async fn delete_user(&self, id: Uuid) -> Result<(), String> {
        let mut store = self.store.lock().await;
        if let Some(pos) = store.users.iter().position(|u| u.id() == id) {
            store.users.remove(pos);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
}

pub fn create_user_repo() -> Arc<UserMemRepo> {
    Arc::new(UserMemRepo {
        store: Arc::new(Mutex::new(UserStore { users: Vec::new() })),
    })
}
