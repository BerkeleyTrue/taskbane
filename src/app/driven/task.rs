use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use derive_more::Constructor;
use taskchampion::{storage::Storage, Task};

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
            .all_tasks()
            .await
            .map(|hm| hm.into_values().collect::<Vec<Task>>())?;
        Ok(tasks)
    }
}

pub fn create_task_repo<S: Storage>(replica: ArcMutRep<S>) -> Arc<TaskRepo<S>> {
    Arc::new(TaskRepo::new(replica))
}
