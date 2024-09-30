#![cfg(feature = "sqlite")]
use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    let db = sqlite().await;
    App::new().route("/", get(index)).inject(db).start().await;
}

async fn index(Extension(db): Extension<Pool<Sqlite>>) -> AppResult<impl IntoResponse> {
    let res = sqlx::query!("pragma integrity_check")
        .fetch_one(&db)
        .await?;
    Ok(res.integrity_check.unwrap_or("Bad check".to_string()))
}
