use std::sync::Arc;

use anyhow::Result;
use derive_more::Constructor;
use taskchampion::Task;

use crate::{core::ports::TaskRepository, infra::task::ServerConf};

#[derive(Constructor)]
pub struct TaskService {
    repo: Arc<dyn TaskRepository>,
    server_config: ServerConf,
}

impl TaskService {
    pub async fn list(&self) -> Result<Vec<Task>> {
        self.repo.list().await
    }

    pub async fn sync(&mut self) -> Result<()> {
        let mut server = self.server_config.into_server().await?;
        self.repo.sync(&mut server).await
    }
}
