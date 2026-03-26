use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use derive_more::Constructor;
use taskchampion::{storage::Storage, Replica, Server, Task};
use tokio::sync::Mutex;
use tracing::info;

use crate::core::ports::TaskRepository;

#[derive(Constructor)]
pub struct TaskRepo<S: Storage + Sync> {
    replica: Mutex<Replica<S>>,
}

#[async_trait]
impl<S: Storage + Sync> TaskRepository for TaskRepo<S> {
    async fn list(&self) -> anyhow::Result<Vec<Task>> {
        let tasks = self
            .replica
            .lock()
            .await
            .all_tasks()
            .await
            .map(|hm| hm.into_values().collect::<Vec<Task>>())?;
        info!("tasks: {tasks:?}");
        Ok(tasks)
    }

    async fn sync(&self, server: &mut Box<dyn Server>) -> Result<()> {
        self.replica.lock().await.sync(server, false).await?;

        Ok(())
    }
}

pub fn create_task_repo<S: Storage + Sync>(replica: Replica<S>) -> Arc<TaskRepo<S>> {
    Arc::new(TaskRepo::new(Mutex::new(replica)))
}
