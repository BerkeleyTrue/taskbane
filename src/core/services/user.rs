use crate::core::ports::user as port;

pub struct UserService {
    repo: Box<dyn port::UserRepository>,
}

impl UserService {
    pub fn new(repo: Box<dyn port::UserRepository>) -> Self {
        Self {
            repo: repo
        }
    }
}
