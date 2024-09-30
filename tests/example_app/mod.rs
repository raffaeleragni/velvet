use velvet_web::prelude::*;

// porting some tests from veltes once test support in in velvet lib direcly
pub async fn app() -> AppResult<App> {
    #[derive(RustEmbed)]
    #[folder = "tests/statics"]
    struct S;

    #[cfg(feature = "auth")]
    JWT::Secret.setup().await?;

    #[cfg(feature = "sqlite")]
    let db = sqlite().await;
    #[cfg(feature = "sqlite")]
    sqlx::migrate!().run(&db).await?;

    let app = App::new()
        .route("/", get(index))
        .inject(client())
        .statics::<S>();
    #[cfg(feature = "sqlite")]
    let app = app.inject(db);
    Ok(app)
}

#[axum::debug_handler]
async fn index() -> impl IntoResponse {
    "OK"
}
