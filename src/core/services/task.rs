use std::sync::Arc;

use anyhow::Result;
use derive_more::Constructor;
use itertools::Itertools;
use taskchampion::Task;

use crate::core::{models::task::TaskDto, ports::TaskRepository};

#[derive(Constructor, Clone)]
pub struct TaskService {
    repo: Arc<dyn TaskRepository>,
}

impl TaskService {
    pub async fn list(&self) -> Result<Vec<TaskDto>> {
        let tasks = self
            .repo
            .list()
            .await?
            .into_iter()
            .map(|(id, task, deps)| TaskDto::from(id, task, deps))
            .sorted_by_key(|task| -(task.urgency * 100.) as i64)
            .collect();
        Ok(tasks)
    }
    pub async fn get_authorize_task(&self) -> Result<Task> {
        self.repo
            .find(&|task| task.get_description().starts_with("taskbane:"))
            .await
            .and_then(|maybe_task| maybe_task.ok_or(anyhow::anyhow!("No authorizing task found")))
    }
}
