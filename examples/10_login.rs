#![cfg(all(feature = "sqlite", feature = "auth", feature = "login"))]

use velvet_web::prelude::*;

#[derive(Deserialize)]
struct Claims;

#[tokio::main]
async fn main() -> AppResult<()> {
    let db = sqlite().await;
    sqlx::migrate!().run(&db).await?;
    JWT::Secret.setup().await?;
    login_setup(&db).await?;
    let router = Router::new()
        .route("/", get(index))
        // everything above this authorized method will require auth
        .authorized_cookie_claims("/login", |_: Claims| Ok(AuthResult::OK))
        .route("/register", get(register))
        .route("/confirm/:user/:code", get(confirm))
        .route("/login", get(login))
        .route("/logout", get(logout));
    App::new().router(router).inject(db).start().await.unwrap();
    Ok(())
}

async fn index() -> impl IntoResponse {
    "Hello World"
}

#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn register(
    Extension(db): Extension<Pool<Sqlite>>,
    Form(register_form): Form<RegisterForm>,
) -> AppResult<String> {
    let confirmation_code = register_user(
        &db,
        &register_form.username,
        &register_form.email,
        &register_form.password,
    )
    .await?;
    Ok(confirmation_code)
}

async fn confirm(
    Extension(db): Extension<Pool<Sqlite>>,
    Path(user): Path<String>,
    Path(code): Path<String>,
) -> AppResult<Redirect> {
    register_user_confirm(&db, &user, &code).await?;
    Ok(Redirect::to("/login"))
}

async fn login(
    Extension(db): Extension<Pool<Sqlite>>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> AppResult<(CookieJar, Redirect)> {
    login_cookie(jar, "/", &db, &form.username, &form.password).await
}

async fn logout(jar: CookieJar) -> AppResult<(CookieJar, Redirect)> {
    logout_cookie(jar, "/login")
}
