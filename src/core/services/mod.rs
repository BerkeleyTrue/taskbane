mod auth;
mod task;
mod user;

use std::sync::Arc;

use webauthn_rs::Webauthn;

use crate::{core::ports, types::ArcMut};

pub use auth::AuthService;
pub use task::TaskService;
pub use user::UserService;

pub struct CreateServiceParams {
    pub user_repo: Arc<dyn ports::UserRepository>,
    pub auth_repo: Arc<dyn ports::AuthRepository>,
    pub task_repo: Arc<dyn ports::TaskRepository>,
    pub webauthn: Arc<Webauthn>,
}

pub fn create_services(
    CreateServiceParams {
        user_repo,
        auth_repo,
        task_repo,
        webauthn,
    }: CreateServiceParams,
) -> (
    user::UserService,
    ArcMut<task::TaskService>,
    auth::AuthService,
) {
    (
        user::UserService::new(user_repo),
        task::TaskService::new(task_repo),
        auth::AuthService::new(auth_repo, webauthn),
    )
}
