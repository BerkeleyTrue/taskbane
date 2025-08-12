use crate::core::{models::User, ports::user as port};
use async_trait::async_trait;
use uuid::Uuid;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub struct UserStore {
    users: HashMap<Uuid, User>,
}

pub struct UserMemRepo {
    store: Arc<Mutex<UserStore>>,
}

#[async_trait]
impl port::UserRepository for UserMemRepo {
    async fn add_user(&self, create_user: port::CreateUser) -> Result<User, String> {
        let mut store = self.store.lock().await;
        let new_user_id = create_user.id.clone();
        let existing_user = store.users.iter().any(move |(uuid, _)| {
            uuid.clone() == new_user_id
        });

        if existing_user {
            return Err("User with username already exists".to_string());
        }

        let user = User::new(create_user.id, create_user.username);
        let user_clone = user.clone();
        store.users.insert(create_user.id, user_clone);

        Ok(user)
    }

    async fn get_user(&self, id: Uuid) -> Result<User, String> {
        let store = self.store.lock().await;

        let Some(user) = store.users.get(&id) else {
            return Err("User not found".to_string());
        };

        Ok(user.clone())
    }

    async fn update_user(&self, user: port::UpdateUser) -> Result<(), String> {
        let mut store = self.store.lock().await;

        if let Some(existing_user) = store.users.get_mut(&user.id) {
            existing_user.with_username(user.username);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    async fn delete_user(&self, id: Uuid) -> Result<(), String> {
        let mut store = self.store.lock().await;
        if let Some(_) = store.users.remove(&id) {
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
}

pub fn create_user_repo() -> Arc<UserMemRepo> {
    Arc::new(UserMemRepo {
        store: Arc::new(Mutex::new(UserStore { users: HashMap::new() })),
    })
}
