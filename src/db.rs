use std::env;

use sqlx::{postgres::PgPoolOptions, PgPool};

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
