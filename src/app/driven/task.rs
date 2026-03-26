use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use derive_more::Constructor;
use taskchampion::{storage::Storage, Status, Task, WorkingSet};

use crate::{core::ports::TaskRepository, infra::task::ArcMutRep};

#[derive(Constructor, Clone)]
pub struct TaskRepo<S: Storage> {
    replica: ArcMutRep<S>,
}

#[async_trait]
impl<S: Storage> TaskRepository for TaskRepo<S> {
    async fn list(&self) -> Result<(Vec<Task>, WorkingSet)> {
        let mut rep = self.replica.lock().await;
        let tasks = rep.pending_tasks().await.map(|tasks| {
            tasks
                .into_iter()
                .filter(|task| task.get_status() == Status::Pending && !task.is_waiting())
                .collect()
        })?;

        let ws = rep.working_set().await?;

        Ok((tasks, ws))
    }
}

pub fn create_task_repo<S: Storage>(replica: ArcMutRep<S>) -> Arc<TaskRepo<S>> {
    Arc::new(TaskRepo::new(replica))
}
