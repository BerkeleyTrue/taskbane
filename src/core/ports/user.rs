use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::core::models::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn add(&self, id: Uuid, username: String) -> Result<User>;
    async fn get(&self, id: Uuid) -> Result<User>;
    async fn get_by_username(&self, username: String) -> Option<User>;
    async fn update(&self, id: Uuid, username: String) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}
