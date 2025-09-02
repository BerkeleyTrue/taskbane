use taskchampion::storage::{InMemoryStorage, Storage};

pub fn create_task_storage() -> impl Storage {
    InMemoryStorage::new()
}
