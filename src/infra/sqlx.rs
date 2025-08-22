use std::{env, str::FromStr};

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tracing::info;

pub fn create_sqlx() -> SqlitePool {
    let db_url = &env::var("DB_URL").expect("No db url provided").to_string();
    info!("setup sqlite with database at {}", db_url);
    let ops = SqliteConnectOptions::from_str(db_url)
        .unwrap()
        .create_if_missing(true);
    SqlitePool::connect_lazy_with(ops)
}
