use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::core::models::user::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn add(&self, id: Uuid, username: &str) -> Result<User>;
    async fn get(&self, id: Uuid) -> Result<User>;
    async fn get_by_username(&self, username: &str) -> Option<User>;
    async fn update(&self, id: Uuid, username: &str) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}
