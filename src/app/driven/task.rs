use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use derive_more::Constructor;
use taskchampion::{storage::Storage, Status, Task};

use crate::{core::ports::TaskRepository, infra::task::ArcMutRep};

#[derive(Constructor)]
pub struct TaskRepo<S: Storage> {
    replica: ArcMutRep<S>,
}

#[async_trait]
impl<S: Storage + Sync> TaskRepository for TaskRepo<S> {
    async fn list(&self) -> Result<Vec<Task>> {
        let tasks = self
            .replica
            .lock()
            .await
            .pending_tasks()
            .await
            .map(|tasks| {
                tasks
                    .into_iter()
                    .filter(|task| task.get_status() == Status::Pending && !task.is_waiting())
                    .collect()
            })?;
        Ok(tasks)
    }
}

pub fn create_task_repo<S: Storage>(replica: ArcMutRep<S>) -> Arc<TaskRepo<S>> {
    Arc::new(TaskRepo::new(replica))
}
