use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use derive_more::Constructor;
use taskchampion::{storage::Storage, Operations, Status, Task};
use uuid::Uuid;

use crate::{core::ports::TaskRepository, infra::task::ArcRep};

#[derive(Constructor, Clone)]
pub struct TaskRepo<S: Storage + Sync> {
    replica: ArcRep<S>,
}

#[async_trait]
impl<S: Storage + Sync> TaskRepository for TaskRepo<S> {
    async fn list(&self) -> Result<Vec<(usize, Task, Vec<usize>)>> {
        let mut rep = self.replica.write().await;
        let ws = rep.working_set().await?;
        let tasks = rep.pending_tasks().await.map(|tasks| {
            tasks
                .into_iter()
                .filter(|task| task.get_status() == Status::Pending && !task.is_waiting())
                .filter_map(move |task| {
                    let deps = task
                        .get_dependencies()
                        .filter_map(|uuid| ws.by_uuid(uuid))
                        .collect();
                    ws.by_uuid(task.get_uuid()).map(move |id| (id, task, deps))
                })
                .collect()
        })?;

        Ok(tasks)
    }
    async fn find(
        &self,
        filter: &(dyn for<'a> Fn(&'a Task) -> bool + Send + Sync),
    ) -> Result<Option<Task>> {
        let mut rep = self.replica.write().await;
        let res = rep.pending_tasks().await?.into_iter().find(filter);

        Ok(res)
    }

    async fn mark_task_done(&self, uuid: Uuid) -> Result<()> {
        let mut rep = self.replica.write().await;
        let mut ops = Operations::new();
        let mut task = rep.get_task(uuid).await?.ok_or(anyhow!("No task found"))?;

        task.done(&mut ops)?;

        rep.commit_operations(ops).await?;

        Ok(())
    }
}

pub fn create_task_repo<S: Storage + Sync>(replica: ArcRep<S>) -> Arc<TaskRepo<S>> {
    Arc::new(TaskRepo::new(replica))
}
