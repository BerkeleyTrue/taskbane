use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use derive_more::Constructor;
use taskchampion::{storage::Storage, Status};

use crate::{
    core::{models::TaskDto, ports::TaskRepository},
    infra::task::ArcRep,
};

#[derive(Constructor, Clone)]
pub struct TaskRepo<S: Storage + Sync> {
    replica: ArcRep<S>,
}

#[async_trait]
impl<S: Storage + Sync> TaskRepository for TaskRepo<S> {
    async fn list(&self) -> Result<Vec<TaskDto>> {
        let mut rep = self.replica.write().await;
        let ws = rep.working_set().await?;
        let tasks = rep.pending_tasks().await.map(|tasks| {
            tasks
                .into_iter()
                .filter(|task| task.get_status() == Status::Pending && !task.is_waiting())
                .filter_map(move |task| ws.by_uuid(task.get_uuid()).map(move |id| (id, task)))
                .map(|(id, task)| TaskDto::from(id, task))
                .collect()
        })?;

        Ok(tasks)
    }
}

pub fn create_task_repo<S: Storage + Sync>(replica: ArcRep<S>) -> Arc<TaskRepo<S>> {
    Arc::new(TaskRepo::new(replica))
}
