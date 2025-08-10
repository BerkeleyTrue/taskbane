#[derive(Debug, Clone)]
pub struct User {
    id: u32,
    username: String,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl User {
    pub fn new(id: u32, username: String) -> Self {
        User { id, username }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn with_username(&mut self, username: String) -> &mut Self {
        self.username = username;
        self
    }
}
