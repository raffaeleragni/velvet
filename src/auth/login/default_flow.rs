use super::{login_cookie, login_setup, logout_cookie, register_user, register_user_confirm, DB};
use crate::{
    app::App,
    prelude::{AppResult, JWT},
};
use askama::Template;
use axum::{
    response::{IntoResponse, Redirect},
    routing::get,
    Extension, Form, Router,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

#[derive(Debug, Default)]
pub struct LoginConfig {}

pub async fn add_default_flow(db: &DB, _config: LoginConfig, app: App) -> App {
    JWT::Secret.setup().await.expect("JWT initialization error");
    login_setup(db).await.expect("Login initialization error");
    let router = Router::new()
        .route("/register", get(register_form).post(register))
        .route("/login", get(login_form).post(login))
        .route("/logout", get(logout));
    app.router(router)
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

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate;

async fn login_form() -> impl IntoResponse {
    LoginTemplate
}

async fn register_form() -> impl IntoResponse {
    RegisterTemplate
}

async fn register(
    Extension(db): Extension<DB>,
    Form(register_form): Form<RegisterForm>,
) -> AppResult<Redirect> {
    let confirmation_code = register_user(
        &db,
        &register_form.username,
        &register_form.email,
        &register_form.password,
    )
    .await?;
    register_user_confirm(&db, &register_form.username, &confirmation_code).await?;
    Ok(Redirect::to("/login"))
}

async fn login(
    Extension(db): Extension<DB>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> AppResult<(CookieJar, Redirect)> {
    login_cookie(jar, "/", &db, &form.username, &form.password).await
}

async fn logout(jar: CookieJar) -> AppResult<(CookieJar, Redirect)> {
    logout_cookie(jar, "/login")
}
