
use std::sync::Arc;

use async_trait::async_trait;
use taskchampion::{storage::Storage, Replica, Task};

use crate::core::ports::TaskRepository;

pub struct TaskRepo<S: Storage> {
    replica: Replica<S>,
}


#[async_trait]
impl<S: Storage> TaskRepository for TaskRepo<S> {
    async fn list(&mut self) -> Result<Vec<Task>, String> {
        self.replica
            .all_tasks()
            .await
            .map_err(|err| err.to_string())
            .map(|hm| hm.into_values().collect::<Vec<Task>>())
    }
}

pub fn create_task_repo<S: Storage>(task_storage: S) -> Arc<TaskRepo<S>> {
    let replica = Replica::new(task_storage);
    Arc::new(TaskRepo { replica })
}
