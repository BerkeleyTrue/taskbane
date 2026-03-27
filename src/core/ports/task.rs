use anyhow::Result;
use async_trait::async_trait;

use crate::core::models::TaskDto;

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<TaskDto>>;
}
