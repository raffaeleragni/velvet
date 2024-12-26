use super::{login_cookie, login_setup, logout_cookie, register_user, register_user_confirm, DB};
use crate::{
    app::App,
    mail::send_confirmation_email,
    prelude::{AppResult, JWT},
};
use askama::Template;
use axum::{
    extract::Query,
    http::Uri,
    response::{IntoResponse, Redirect},
    routing::get,
    Extension, Form, Router,
};
use axum_extra::extract::CookieJar;
use lettre::SmtpTransport;
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

pub async fn add_mail_flow(db: &DB, _config: LoginConfig, app: App) -> App {
    JWT::Secret.setup().await.expect("JWT initialization error");
    login_setup(db).await.expect("Login initialization error");
    let router = Router::new()
        .route("/register", get(register_form).post(register_send_mail))
        .route("/confirm", get(confirm_form).post(confirm))
        .route("/login", get(login_form).post(login))
        .route("/logout", get(logout));
    app.router(router).inject(mailer())
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

#[derive(Deserialize)]
struct ConfirmForm {
    username: String,
    code: String,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate;

#[derive(Template)]
#[template(path = "confirm.html")]
struct ConfirmTemplate {
    username: String,
}

async fn login_form() -> impl IntoResponse {
    LoginTemplate
}

async fn register_form() -> impl IntoResponse {
    RegisterTemplate
}

#[derive(Deserialize)]
struct ConfirmQuery {
    username: String,
}

async fn confirm_form(Query(q): Query<ConfirmQuery>) -> impl IntoResponse {
    ConfirmTemplate {
        username: q.username,
    }
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

async fn register_send_mail(
    Extension(mailer): Extension<SmtpTransport>,
    Extension(db): Extension<DB>,
    url: Uri,
    Form(register_form): Form<RegisterForm>,
) -> AppResult<Redirect> {
    let confirmation_code = register_user(
        &db,
        &register_form.username,
        &register_form.email,
        &register_form.password,
    )
    .await?;
    let scheme = url
        .scheme()
        .map(|s| format!("{}://", s))
        .unwrap_or_default();
    let base = format!(
        "{}{}",
        scheme,
        url.authority().map(|s| s.to_string()).unwrap_or_default()
    );
    let link = format!("{}/confirm?username={}", base, register_form.username);
    send_confirmation_email(
        mailer,
        &base,
        &link,
        &register_form.username,
        &register_form.email,
        &confirmation_code,
    )?;
    Ok(Redirect::to(
        format!("/confirm?username={}", &register_form.username).as_str(),
    ))
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

async fn confirm(
    Extension(db): Extension<DB>,
    Form(form): Form<ConfirmForm>,
) -> AppResult<impl IntoResponse> {
    register_user_confirm(&db, &form.username, &form.code).await?;
    Ok(Redirect::to("/login"))
}
