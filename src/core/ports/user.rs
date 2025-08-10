use async_trait::async_trait;

use crate::core::models::User;

pub struct CreateUser {
    pub id: u32,
    pub username: String,
}

pub struct UpdateUser {
    pub id: u32,
    pub username: String,
}

#[async_trait]
pub trait UserRepository {
    async fn add_user(&self, user: CreateUser);
    async fn get_user(&self, id: u32) -> Result<User, String>;
    async fn update_user(&self, user: UpdateUser) -> Result<(), String>;
    async fn delete_user(&self, id: u32) -> Result<(), String>;
}
