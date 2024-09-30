#[cfg(feature = "postgres")]
pub async fn postgres() -> sqlx::PgPool {
    // May not know if app is constructed before databse, so trigger dotenvs in both situations
    dotenvy::dotenv().ok();
    crate::app::logger();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");
    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .unwrap()
}

#[cfg(feature = "sqlite")]
pub async fn sqlite() -> sqlx::SqlitePool {
    dotenvy::dotenv().ok();
    crate::app::logger();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");
    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .unwrap()
}

#[cfg(feature = "mysql")]
pub async fn mysql() -> sqlx::MySqlPool {
    dotenvy::dotenv().ok();
    crate::app::logger();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");
    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .unwrap()
}
