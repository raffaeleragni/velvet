#![cfg(all(feature = "sqlite", feature = "auth", feature = "login"))]

use velvet_web::prelude::*;

#[derive(Deserialize)]
#[allow(dead_code)]
struct Claims {
    username: String,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let db = sqlite().await;
    sqlx::migrate!().run(&db).await?;
    JWT::Secret.setup().await?;
    login_setup(&db).await?;
    let router = Router::new()
        .route("/", get(index))
        // everything above this authorized method will require auth
        .authorized_cookie_claims("/login", |_: Claims| Ok(AuthResult::OK));
    App::new()
        .router(router)
        .login_flow(&db)
        .await
        .inject(db)
        .start()
        .await
        .unwrap();
    Ok(())
}

async fn index() -> impl IntoResponse {
    "Hello World"
}