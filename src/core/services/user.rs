use crate::core::models;
use crate::core::ports;
use anyhow::{anyhow, Result};
use derive_more::Constructor;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Constructor)]
pub struct UserService {
    repo: Arc<dyn ports::user::UserRepository>,
}

impl UserService {
    pub async fn is_username_available(&self, username: &str) -> bool {
        self.repo.get_by_username(username).await.is_none()
    }

    pub async fn register_user(&self, username: &str) -> Result<models::user::User> {
        let id = Uuid::new_v4();
        if !self.is_username_available(username).await {
            return Err(anyhow!("Username already exists"));
        }
        self.repo.add(id, username).await
    }
    pub async fn get_user(&self, username: &str) -> Result<models::user::User> {
        self.repo
            .get_by_username(username)
            .await
            .ok_or(anyhow!("No user found for username"))
    }
}
