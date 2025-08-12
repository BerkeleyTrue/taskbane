use async_trait::async_trait;
use uuid::Uuid;

use crate::core::models::User;

pub struct CreateUser {
    pub id: Uuid,
    pub username: String,
}

pub struct UpdateUser {
    pub id: Uuid,
    pub username: String,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn add(&self, user: CreateUser) -> Result<User, String>;
    async fn get(&self, id: Uuid) -> Result<User, String>;
    async fn get_by_username(&self, username: String) -> Option<User>;
    async fn update(&self, user: UpdateUser) -> Result<(), String>;
    async fn delete(&self, id: Uuid) -> Result<(), String>;
}
