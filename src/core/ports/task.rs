use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use taskchampion::{Tag, Task};
use uuid::Uuid;

#[derive(Debug, Constructor)]
pub struct CreateTaskInput {
    pub description: String,
    pub priority: String,
    pub deps: Vec<Uuid>,
    pub tags: Vec<Tag>,
    pub due: Option<DateTime<Utc>>,
}

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
    async fn create_task(&self, input: CreateTaskInput) -> Result<usize>;
}
