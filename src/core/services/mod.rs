pub mod user;

use crate::core::ports;

pub struct CreateServiceParams {
    pub user_repo: Box<dyn ports::user::UserRepository>,
}

pub fn create_services(params: CreateServiceParams) -> user::UserService {
   user::UserService::new(params.user_repo)
}
