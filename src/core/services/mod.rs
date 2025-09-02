mod auth;
mod task;
mod user;

use std::sync::Arc;

use webauthn_rs::Webauthn;

use crate::core::ports;

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
    params: CreateServiceParams,
) -> (user::UserService, task::TaskService, auth::AuthService) {
    (
        user::UserService::new(params.user_repo),
        task::TaskService::new(params.task_repo),
        auth::AuthService::new(params.auth_repo, params.webauthn),
    )
}
