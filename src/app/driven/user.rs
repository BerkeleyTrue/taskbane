use crate::core::{models::User, ports::user as port};
use async_trait::async_trait;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

pub struct UserSqlRepo {
    pool: SqlitePool,
}

#[async_trait]
impl port::UserRepository for UserSqlRepo {
    async fn add(&self, create_user: port::CreateUser) -> Result<User, String> {
        let new_user_id = create_user.id.clone();
        let existing_user = sqlx::query_as!(
            User,
            r#"SELECT id as `id:uuid::Uuid`, username FROM users WHERE id == ?"#,
            new_user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| err.to_string())?
        .is_some();

        if existing_user {
            return Err("User with username already exists".to_string());
        }

        let user = User::new(create_user.id, create_user.username);
        let username_copy = user.username();
        sqlx::query!(
            r#"
            INSERT INTO users (id, username)
            VALUES (?, ?)
            RETURNING id as `id:uuid::Uuid`, username
        "#,
            new_user_id,
            username_copy,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| err.to_string())?;

        Ok(user)
    }

    async fn get(&self, id: Uuid) -> Result<User, String> {
        sqlx::query_as!(User, "SELECT id as `id:uuid::Uuid`, username FROM users WHERE id == ?", id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| err.to_string())?
            .ok_or("User not found".to_string())
    }

    async fn get_by_username(&self, username: String) -> Option<User> {
        sqlx::query_as!(
            User,
            "SELECT id as `id:uuid::Uuid`, username FROM users WHERE username == ?",
            username
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    async fn update(&self, user: port::UpdateUser) -> Result<(), String> {
        sqlx::query!(
            r#"
                UPDATE users
                SET username = ?
                WHERE id = ?
            "#,
            user.username,
            user.id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| err.to_string())
        .map(|_| ())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        sqlx::query!("DELETE FROM users WHERE id = ?", id)
            .execute(&self.pool)
            .await
            .map_err(|err| err.to_string())
            .map(|_| ())
    }
}

pub fn create_user_repo(pool: &SqlitePool) -> Arc<UserSqlRepo> {
    Arc::new(UserSqlRepo { pool: pool.clone() })
}
