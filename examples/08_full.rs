#![cfg(all(feature = "sqlite", feature = "auth"))]
use std::time::{SystemTime, UNIX_EPOCH};

use velvet_web::prelude::*;

#[tokio::main]
async fn main() -> AppResult<()> {
    #[derive(RustEmbed)]
    #[folder = "statics"]
    struct S;
    JWT::Secret.setup().await?;
    let db = sqlite().await;
    sqlx::migrate!().run(&db).await?;

    let router = Router::new()
        .route("/", get(index))
        .authorized_cookie_claims(|claims: Claims| Ok(claims.role == "user"))
        .route("/login", get(login));
    App::new()
        .router(router)
        .inject(db)
        .inject(client())
        .statics::<S>()
        .start()
        .await;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: u64,
    role: String,
}

async fn login(jar: CookieJar) -> AppResult<(CookieJar, Redirect)> {
    let jar = CookieToken::set_from_claims(
        jar,
        Claims {
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 3600,
            role: "user".to_string(),
        },
    )
    .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    Ok((jar, Redirect::to("/")))
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index;

async fn index(Extension(db): Extension<Pool<Sqlite>>) -> AppResult<impl IntoResponse> {
    let _ = query!("pragma integrity_check")
        .fetch_one(&db)
        .await?
        .integrity_check;
    Ok(Index {})
}
