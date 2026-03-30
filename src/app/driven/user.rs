use crate::core::{models::user::User, ports::UserRepository};
use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct UserSqlRepo {
    pool: SqlitePool,
}

#[async_trait]
impl UserRepository for UserSqlRepo {
    async fn add(&self, id: Uuid, username: String) -> Result<User> {
        let existing_user = sqlx::query_as!(
            User,
            r#"SELECT id as `id:uuid::Uuid`, username FROM users WHERE id == ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .is_some();

        if existing_user {
            return Err(anyhow!("User with username already exists"));
        }

        let user = User::new(id, username);
        let username_copy = user.username();
        sqlx::query!(
            r#"
            INSERT INTO users (id, username)
            VALUES (?, ?)
            RETURNING id as `id:uuid::Uuid`, username
        "#,
            id,
            username_copy,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn get(&self, id: Uuid) -> Result<User> {
        sqlx::query_as!(
            User,
            "SELECT id as `id:uuid::Uuid`, username FROM users WHERE id == ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(anyhow!("User not found"))
    }

    async fn get_by_username(&self, username: String) -> Option<User> {
        sqlx::query_as!(
            User,
            "SELECT id as `id:uuid::Uuid`, username FROM users WHERE username == ?",
            username
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|err| {
            info!("Error fetching user by username: {}", err);
        })
        .unwrap_or(None)
    }

    async fn update(&self, id: Uuid, username: String) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE users
                SET username = ?
                WHERE id = ?
            "#,
            username,
            id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::from)
        .map(|_| ())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM users WHERE id = ?", id)
            .execute(&self.pool)
            .await
            .map_err(Error::from)
            .map(|_| ())
    }
}

pub fn create_user_repo(pool: &SqlitePool) -> Arc<UserSqlRepo> {
    Arc::new(UserSqlRepo { pool: pool.clone() })
}
