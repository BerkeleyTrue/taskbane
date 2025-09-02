use std::sync::Arc;

use taskchampion::Task;

use crate::core::ports::TaskRepository;

#[derive(Clone)]
pub struct TaskService {
    repo: Arc<dyn TaskRepository>,
}

impl TaskService {
    pub fn new(repo: Arc<dyn TaskRepository>) -> Self {
        Self {
            repo
        }
    }

    pub async fn list(&self) -> Result<Vec<Task>, String> {
        return Ok(Vec::new())
    }
}

