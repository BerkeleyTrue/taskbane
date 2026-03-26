use anyhow::Result;
use async_trait::async_trait;
use taskchampion::{Server, Task};

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Task>>;
    async fn sync(&self, server: &mut Box<dyn Server>) -> Result<()>;
}
