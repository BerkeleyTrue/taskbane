use std::{env, str::FromStr};

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    SqlitePool,
};
use tracing::info;

pub fn create_sqlx() -> SqlitePool {
    let db_url = &env::var("DB_URL").expect("No db url provided").to_string();
    info!("setup sqlite with database at {}", db_url);
    let ops = SqliteConnectOptions::from_str(db_url)
        .unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    SqlitePool::connect_lazy_with(ops)
}

pub async fn run_migration(pool: &SqlitePool) -> Result<()> {
    info!("running migrations");
    sqlx::migrate!().run(pool).await?;
    Ok(())
}
