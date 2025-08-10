pub mod auth;
pub mod user;

use crate::core::ports::user as port;

pub fn create_driven() -> Box<dyn port::UserRepository> {
    user::create_user_repo()
}
