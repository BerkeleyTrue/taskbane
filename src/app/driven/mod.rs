pub mod user;

use crate::core::ports::user as port;

pub fn create_driven() -> impl port::UserRepository {
    user::create_user_repo()
}
