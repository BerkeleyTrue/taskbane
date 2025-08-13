pub mod auth;
pub mod user;

use std::sync::Arc;

use crate::core::ports::user as port;

pub fn create_driven() -> (Arc<dyn port::UserRepository>, auth::ChallengeService) {
    (user::create_user_repo(), auth::create_challenge_service())
}
