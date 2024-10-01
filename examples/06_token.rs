#![cfg(feature = "auth")]
use velvet_web::prelude::*;

#[derive(Deserialize)]
struct Claims {
    role: String,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    JWT::Secret.setup().await?;
    let router = Router::new()
        .route("/", get(index))
        .authorized_bearer_claims(|claims: Claims| Ok(claims.role == "admin"));
    App::new().router(router).start().await.unwrap();
    Ok(())
}

async fn index() -> AppResult<impl IntoResponse> {
    Ok("Hello World")
}
