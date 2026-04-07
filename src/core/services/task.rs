use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDateTime};
use derive_more::Constructor;
use itertools::Itertools;
use taskchampion::Task;
use uuid::Uuid;

use crate::{
    core::{models::task::TaskDto, ports::TaskRepository},
    infra::datetime::parse_date,
};

#[derive(Constructor, Clone)]
pub struct TaskService {
    repo: Arc<dyn TaskRepository>,
}

impl TaskService {
    pub async fn get_task(&self, uuid: Uuid) -> Result<TaskDto> {
        let task = self
            .repo
            .get_task(uuid)
            .await?
            .ok_or(anyhow!("No task found for uuid"))?;

        let deps = task.get_dependencies().collect::<Vec<Uuid>>();
        let (id, deps) = self.repo.get_task_meta(task.get_uuid(), deps).await?;

        Ok(TaskDto::from(id, task, deps))
    }
    pub async fn list(&self) -> Result<Vec<TaskDto>> {
        let tasks = self
            .repo
            .list()
            .await?
            .into_iter()
            .map(|(id, task, deps)| TaskDto::from(id, task, deps))
            .sorted_by_key(|task| -(task.urgency * 100.) as i64)
            .collect();
        Ok(tasks)
    }
    pub async fn get_authorize_task(&self) -> Result<Task> {
        self.repo
            .find(&|task| task.get_description().starts_with("taskbane:"))
            .await
            .and_then(|maybe_task| maybe_task.ok_or(anyhow::anyhow!("No authorizing task found")))
    }

    pub async fn mark_task_done(&self, uuid: Uuid) -> Result<()> {
        self.repo.mark_task_done(uuid).await
    }

    pub fn parse_datetime(&self, due: &str) -> Result<NaiveDateTime> {
        parse_date(due, Local::now().naive_local()).ok_or_else(|| anyhow!("Could not parse"))
    }
}
