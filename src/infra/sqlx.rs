use std::env;

use sqlx::{SqlitePool};


pub fn create_sqlx() -> SqlitePool {
    let db_url = &env::var("DB_URL").expect("No db url provided").to_string();
    SqlitePool::connect_lazy(db_url).unwrap()
}
