use crate::core::models;
use crate::core::ports;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    repo: Arc<dyn ports::UserRepository>,
}

impl UserService {
    pub fn new(repo: Arc<dyn ports::UserRepository>) -> Self {
        Self { repo: repo }
    }

    pub async fn is_username_available(&self, username: String) -> bool {
        self.repo.get_by_username(username).await.is_none()
    }

    pub async fn register_user(&self, username: String) -> Result<models::User, String> {
        let id = Uuid::new_v4();
        if !self.is_username_available(username.clone()).await {
            return Err("Username already exists".to_string());
        }
        self.repo.add(id, username).await
    }
    pub async fn get_login(&self, username: String) -> Result<models::User, String> {
        self.repo
            .get_by_username(username)
            .await
            .ok_or_else(|| "No user found for username".to_string())
    }
}
