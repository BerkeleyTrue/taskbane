use anyhow::Result;
use async_trait::async_trait;
use taskchampion::Task;
use uuid::Uuid;

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn get_task(&self, uuid: Uuid) -> Result<Option<Task>>;
    async fn get_task_meta(&self, uuid: Uuid, deps: Vec<Uuid>) -> Result<(usize, Vec<usize>)>;
    async fn list(&self) -> Result<Vec<(usize, Task, Vec<usize>)>>;
    async fn find(
        &self,
        filter: &(dyn for<'a> Fn(&'a Task) -> bool + Send + Sync),
    ) -> Result<Option<Task>>;
    async fn mark_task_done(&self, uuid: Uuid) -> Result<()>;
}
