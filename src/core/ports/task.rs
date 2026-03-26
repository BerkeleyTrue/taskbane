use anyhow::Result;
use async_trait::async_trait;
use taskchampion::{Task, WorkingSet};

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn list(&self) -> Result<(Vec<Task>, WorkingSet)>;
}
