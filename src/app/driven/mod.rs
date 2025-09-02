mod auth;
mod user;

use std::sync::Arc;

use sqlx::SqlitePool;

use crate::core::ports;

pub fn create_driven(
    pool: &SqlitePool
) -> (Arc<dyn ports::UserRepository>, Arc<dyn ports::AuthRepository>) {
    (
        user::create_user_repo(pool),
        auth::create_auth_repo(pool),
    )
}
