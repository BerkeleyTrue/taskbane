mod auth;
mod task;
mod user;

use std::sync::Arc;

use sqlx::SqlitePool;
use taskchampion::storage::Storage;

use crate::{core::ports, infra::task::ArcMutRep};

pub fn create_driven<S: Storage + Sync + 'static>(
    pool: &SqlitePool,
    task_storage: ArcMutRep<S>,
) -> (
    Arc<dyn ports::UserRepository>,
    Arc<dyn ports::AuthRepository>,
    Arc<dyn ports::TaskRepository>,
) {
    (
        user::create_user_repo(pool),
        auth::create_auth_repo(pool),
        task::create_task_repo(task_storage),
    )
}
