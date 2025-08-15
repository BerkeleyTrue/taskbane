pub mod auth;
pub mod user;

use std::sync::Arc;

use webauthn_rs::Webauthn;

use crate::core::ports::user as port;

pub fn create_driven(webauthn: Arc<Webauthn>) -> (Arc<dyn port::UserRepository>, auth::AuthService) {
    (user::create_user_repo(), auth::create_auth_service(webauthn))
}
