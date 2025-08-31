use taskchampion::Task;

#[derive(Clone)]
pub struct TaskService {
}

impl TaskService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn list(&self) -> Result<Vec<Task>, String> {
        return Ok(Vec::new())
    }
}

