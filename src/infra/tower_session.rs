use anyhow::Result;
use sqlx::SqlitePool;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::SqliteStore;

pub trait MySession {
    async fn run_migration(&self) -> Result<()>;
    fn create_layer(self) -> SessionManagerLayer<SqliteStore>;
}

impl MySession for SqliteStore {
    async fn run_migration(&self) -> Result<()> {
        self.migrate().await?;
        Ok(())
    }

    fn create_layer(self) -> SessionManagerLayer<SqliteStore> {
        SessionManagerLayer::new(self).with_secure(false)
    }
}

pub fn create_session_store(pool: &SqlitePool) -> SqliteStore {
    SqliteStore::new(pool.clone())
}
