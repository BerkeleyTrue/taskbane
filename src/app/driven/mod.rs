pub mod auth;
pub mod user;

use std::sync::Arc;

use sqlx::SqlitePool;

use crate::core::ports;

pub fn create_driven(
    pool: &SqlitePool
) -> (Arc<dyn ports::user::UserRepository>, Arc<dyn ports::auth::AuthRepository>) {
    (
        user::create_user_repo(pool),
        auth::create_auth_repo(pool),
    )
}
