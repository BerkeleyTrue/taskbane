use derive_more::{Constructor, Eq, PartialEq};
use serde::Serialize;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow, PartialEq, Eq, Constructor)]
pub struct User {
    pub id: Uuid,
    #[eq(skip)]
    pub username: String,
}

impl User {
    pub fn id(&self) -> Uuid {
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
