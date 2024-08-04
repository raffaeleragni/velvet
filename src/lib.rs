mod app;
mod auth;
mod client;
mod db;
mod errors;

#[macro_use]
pub mod prelude {
    pub use super::app::App;
    pub use super::auth::jwt::claims_for;
    pub use super::auth::jwt::VerifiedClaims;
    pub use super::auth::jwt::JWT;
    pub use super::auth::AuthorizedBearer;
    pub use super::auth::AuthorizedCookie;
    pub use super::auth::BearerToken;
    pub use super::auth::CookieToken;
    pub use super::client::client;
    pub use super::db::mysql;
    pub use super::db::postgres;
    pub use super::db::sqlite;
    pub use super::errors::AppError;
    pub use super::errors::AppResult;
    pub use askama::Template;
    pub use axum::extract::{Form, Json, Path};
    pub use axum::http::StatusCode;
    pub use axum::http::HeaderMap;
    pub use axum::http::HeaderName;
    pub use axum::http::HeaderValue;
    pub use axum::response::Redirect;
    pub use axum::routing::{delete, get, patch, post, put};
    pub use axum::{Extension, Router};
    pub use axum_extra::extract::CookieJar;
    pub use jsonwebtoken::DecodingKey;
    pub use reqwest::Client;
    pub use rust_embed::RustEmbed;
    pub use serde::{Deserialize, Serialize};
    pub use sqlx::{query, query_as, MySql, Pool, Postgres, Sqlite};
    pub use tracing::{debug, error, info, instrument, span, trace, warn, Level};
    pub use valuable::Valuable;
}
