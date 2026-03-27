use std::sync::Arc;

use anyhow::Result;
use derive_more::Constructor;

use crate::core::{models::TaskDto, ports::TaskRepository};

#[derive(Constructor, Clone)]
pub struct TaskService {
    repo: Arc<dyn TaskRepository>,
}

impl TaskService {
    pub async fn list(&mut self) -> Result<Vec<TaskDto>> {
        self.repo.list().await
    }
}
