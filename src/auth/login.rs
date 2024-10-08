#![cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]

use super::{jwt::token_from_claims, CookieToken};
use crate::prelude::AppResult;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2,
};
use argon2::{PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{http::status::StatusCode, response::Redirect};
use axum_extra::extract::CookieJar;
use sentry::types::random_uuid;
use serde::Serialize;
use sqlx::Pool;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::warn;

#[cfg(feature = "sqlite")]
type DB = Pool<sqlx::Sqlite>;

#[cfg(feature = "mysql")]
type DB = Pool<sqlx::Mysql>;

#[cfg(feature = "postgres")]
type DB = Pool<sqlx::Postgres>;

pub async fn login_setup(db: &DB) -> AppResult<()> {
    let create = r#"
create table if not exists login (
    userid varchar(255) not null,
    username varchar(255) not null,
    email varchar(255),
    password varchar(255) not null,
    confirmation_code varchar(255) not null,
    confirmed smallint not null default 0,
    primary key (userid),
    unique (username)
)
"#;
    sqlx::query(create).execute(db).await?;
    Ok(())
}

/// Returns the confirmation code that will be used for register_user_confirm
pub async fn register_user(
    db: &DB,
    username: &str,
    email: &str,
    password: &str,
) -> AppResult<String> {
    let user = User::create(username, email, password)?;
    let code = random_uuid().to_string();
    sqlx::query(
        "insert into login 
        (userid, username, email, password, confirmation_code, confirmed) 
        values(?, ?, ?, ?, ?, 9)",
    )
    .bind(user.userid)
    .bind(user.username)
    .bind(user.email)
    .bind(user.password)
    .bind(code.clone())
    .execute(db)
    .await?;
    Ok(code)
}

pub async fn register_user_confirm(
    db: &DB,
    username: &str,
    confirmation_code: &str,
) -> AppResult<()> {
    sqlx::query("update login set confirmed = 1 where username = ? and confirmation_code = ?")
        .bind(username)
        .bind(confirmation_code)
        .execute(db)
        .await?;
    Ok(())
}

#[derive(Serialize)]
struct Claims {
    exp: u64,
    username: String,
}

async fn login_claims(db: &DB, username: &str, password: &str) -> AppResult<Claims> {
    let row: (String, String) =
        sqlx::query_as("select username, password from login where username = ? and confirmed = 1")
            .bind(username)
            .fetch_one(db)
            .await?;
    let hash = PasswordHash::new(row.1.as_str())?;
    Argon2::default().verify_password(password.as_bytes(), &hash)?;
    Ok(Claims {
        username: username.to_string(),
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600 * 24,
    })
}

pub async fn login_token(db: &DB, username: &str, password: &str) -> AppResult<String> {
    let claims = login_claims(db, username, password).await?;
    token_from_claims(&claims).map_err(|e| {
        warn!(e);
        StatusCode::UNAUTHORIZED.into()
    })
}

pub async fn login_cookie(
    jar: CookieJar,
    redirect: &str,
    db: &DB,
    username: &str,
    password: &str,
) -> AppResult<(CookieJar, Redirect)> {
    let claims = login_claims(db, username, password).await?;
    let jar = CookieToken::set_from_claims(jar, claims).map_err(|e| {
        warn!(e);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;
    Ok((jar, Redirect::to(redirect)))
}

pub fn logout_cookie(jar: CookieJar, redirect: &str) -> AppResult<(CookieJar, Redirect)> {
    let jar = CookieToken::remove(jar);
    Ok((jar, Redirect::to(redirect)))
}

#[derive(Debug, Clone)]
struct User {
    userid: String,
    username: String,
    email: String,
    password: String,
}

impl User {
    fn create(username: &str, email: &str, password: &str) -> AppResult<Self> {
        let salt = SaltString::generate(&mut OsRng);
        let argon = Argon2::default();
        let hash = argon.hash_password(password.as_bytes(), &salt)?.to_string();
        Ok(Self {
            userid: random_uuid().to_string(),
            username: username.to_string(),
            email: email.to_string(),
            password: hash,
        })
    }
}
