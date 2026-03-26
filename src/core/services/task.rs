use std::sync::Arc;

use anyhow::Result;
use taskchampion::{Task, WorkingSet};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{core::ports::TaskRepository, types::ArcMut};

pub struct TaskService {
    repo: Arc<dyn TaskRepository>,
    ws: Option<WorkingSet>,
}

impl TaskService {
    pub fn new(repo: Arc<dyn TaskRepository>) -> ArcMut<Self> {
        Arc::new(Mutex::new(Self { repo, ws: None }))
    }
}

impl TaskService {
    pub async fn list(&mut self) -> Result<Vec<Task>> {
        let (tasks, ws) = self.repo.list().await?;
        self.ws = Some(ws);
        Ok(tasks)
    }

    pub fn get_ws_id(&self, id: Uuid) -> Option<usize> {
        self.ws.as_ref().and_then(move |ws| ws.by_uuid(id))
    }
}
