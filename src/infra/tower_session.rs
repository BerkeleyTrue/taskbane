use sqlx::SqlitePool;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::SqliteStore;

pub trait MySession {
    async fn run_migration(&self) -> Result<(), String>;
    fn create_layer(self) -> SessionManagerLayer<SqliteStore>;
}

impl MySession for SqliteStore {
    async fn run_migration(&self) -> Result<(), String> {
        self.migrate().await.map_err(|err| err.to_string())
    }

    fn create_layer(self) -> SessionManagerLayer<SqliteStore> {
        SessionManagerLayer::new(self).with_secure(false)
    }
}

pub fn create_session_store(pool: SqlitePool) -> SqliteStore {
    SqliteStore::new(pool)
}
