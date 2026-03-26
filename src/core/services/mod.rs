mod auth;
mod task;
mod user;

use std::sync::Arc;

use webauthn_rs::Webauthn;

use crate::{core::ports, infra::task::ServerConf};

pub use auth::AuthService;
pub use task::TaskService;
pub use user::UserService;

pub struct CreateServiceParams {
    pub user_repo: Arc<dyn ports::UserRepository>,
    pub auth_repo: Arc<dyn ports::AuthRepository>,
    pub task_repo: Arc<dyn ports::TaskRepository>,
    pub webauthn: Arc<Webauthn>,
    pub task_server_config: ServerConf,
}

pub fn create_services(
    CreateServiceParams {
        user_repo,
        auth_repo,
        task_repo,
        webauthn,
        task_server_config,
    }: CreateServiceParams,
) -> (user::UserService, task::TaskService, auth::AuthService) {
    (
        user::UserService::new(user_repo),
        task::TaskService::new(task_repo, task_server_config),
        auth::AuthService::new(auth_repo, webauthn),
    )
}
