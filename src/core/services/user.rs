use std::sync::Arc;
use uuid::Uuid;
use crate::core::ports::user as port;
use crate::core::models;

#[derive(Clone)]
pub struct UserService {
    repo: Arc<dyn port::UserRepository>,
}

impl UserService {
    pub fn new(repo: Arc<dyn port::UserRepository>) -> Self {
        Self {
            repo: repo
        }
    }

    pub async fn is_username_available(&self, username: String) -> bool {
        self.repo.get_by_username(username).await.is_none()
    }

    pub async fn register_user(&self, username: String) -> Result<models::User, String> {
        let id = Uuid::new_v4();
        let user = port::CreateUser {
            id,
            username: username.clone(),
        };
        if !self.is_username_available(username.clone()).await {
            return Err("Username already exists".to_string());
        }
        self.repo.add(user).await
    }
}
