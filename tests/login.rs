#![cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
#![cfg(feature = "login")]
#![cfg(feature = "auth")]

use serde::Deserialize;
use serial_test::serial;
use velvet_web::prelude::*;

#[derive(Deserialize)]
struct Claims {
    username: String,
}

#[tokio::test]
#[serial]
async fn test() -> AppResult<()> {
    let db = sqlite().await;
    JWT::Secret.setup().await?;
    login_setup(&db).await?;
    let code = register_user(&db, "user", "email", "password").await?;
    register_user_confirm(&db, "user", &code).await?;
    let token = login_token(&db, "user", "password").await?;
    let claims = claims_for::<Claims>(&token)?;
    assert_eq!(claims.username, "user");
    Ok(())
}
