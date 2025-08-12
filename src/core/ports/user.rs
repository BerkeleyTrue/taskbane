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
    async fn add_user(&self, user: CreateUser) -> Result<User, String>;
    async fn get_user(&self, id: Uuid) -> Result<User, String>;
    async fn update_user(&self, user: UpdateUser) -> Result<(), String>;
    async fn delete_user(&self, id: Uuid) -> Result<(), String>;
}
