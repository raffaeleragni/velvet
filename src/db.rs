use std::env;

use sqlx::{
    mysql::MySqlPoolOptions, postgres::PgPoolOptions, sqlite::SqlitePoolOptions, MySqlPool, PgPool,
    SqlitePool,
};

pub async fn postgres() -> PgPool {
    // May not know if app is constructed before databse, so trigger dotenvs in both situations
    dotenv::dotenv().ok();
    crate::app::logger();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");
    let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .unwrap()
}

pub async fn sqlite() -> SqlitePool {
    dotenv::dotenv().ok();
    crate::app::logger();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");
    let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .unwrap()
}

pub async fn mysql() -> MySqlPool {
    dotenv::dotenv().ok();
    crate::app::logger();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");
    let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    MySqlPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .unwrap()
}
