pub mod auth;
pub mod task;
pub mod user;

use std::sync::Arc;

use webauthn_rs::Webauthn;

use crate::core::ports;

pub struct CreateServiceParams {
    pub user_repo: Arc<dyn ports::UserRepository>,
    pub auth_repo: Arc<dyn ports::AuthRepository>,
    pub webauthn: Arc<Webauthn>,
}

pub fn create_services(
    params: CreateServiceParams,
) -> (user::UserService, task::TaskService, auth::AuthService) {
    (
        user::UserService::new(params.user_repo),
        task::TaskService::new(),
        auth::AuthService::new(params.auth_repo, params.webauthn),
    )
}
