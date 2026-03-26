use std::sync::Arc;

use anyhow::Result;
use derive_more::Constructor;
use taskchampion::Task;

use crate::core::ports::TaskRepository;

#[derive(Constructor, Clone)]
pub struct TaskService {
    repo: Arc<dyn TaskRepository>,
}

impl TaskService {
    pub async fn list(&self) -> Result<Vec<Task>> {
        self.repo.list().await
    }
}
