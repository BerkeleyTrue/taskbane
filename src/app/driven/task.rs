use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Constructor;
use taskchampion::{storage::Storage, Replica, Task};

use crate::core::ports::TaskRepository;

#[derive(Constructor)]
pub struct TaskRepo<S: Storage + Sync> {
    replica: Replica<S>,
}

#[async_trait]
impl<S: Storage + Sync> TaskRepository for TaskRepo<S> {
    async fn list(&mut self) -> anyhow::Result<Vec<Task>> {
        let tasks = self
            .replica
            .all_tasks()
            .await
            .map(|hm| hm.into_values().collect::<Vec<Task>>())?;
        Ok(tasks)
    }
}

pub fn create_task_repo<S: Storage + Sync>(replica: Replica<S>) -> Arc<TaskRepo<S>> {
    Arc::new(TaskRepo::new(replica))
}
