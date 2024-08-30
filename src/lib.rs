mod app;
#[cfg(feature = "auth")]
mod auth;
mod client;
#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
mod db;
mod errors;

#[macro_use]
pub mod prelude {
    pub use super::app::App;
    pub use super::client::client;
    pub use super::errors::AppError;
    pub use super::errors::AppResult;
    pub use askama::Template;
    pub use axum::extract::{Form, Json, Path};
    pub use axum::http::HeaderMap;
    pub use axum::http::HeaderName;
    pub use axum::http::HeaderValue;
    pub use axum::http::StatusCode;
    pub use axum::response::Redirect;
    pub use axum::routing::{delete, get, patch, post, put};
    pub use axum::{Extension, Router};
    pub use reqwest::Client;
    pub use rust_embed::RustEmbed;
    pub use serde::{Deserialize, Serialize};
    pub use sqlx::{query, query_as, Pool};
    pub use tracing::{debug, error, info, instrument, span, trace, warn, Level};
    pub use valuable::Valuable;

    #[cfg(feature = "mysql")]
    pub use super::db::mysql;
    #[cfg(feature = "postgres")]
    pub use super::db::postgres;
    #[cfg(feature = "sqlite")]
    pub use super::db::sqlite;
    #[cfg(feature = "mysql")]
    pub use sqlx::MySql;
    #[cfg(feature = "postgres")]
    pub use sqlx::Postgres;
    #[cfg(feature = "sqlite")]
    pub use sqlx::Sqlite;

    #[cfg(feature = "auth")]
    pub use super::auth::jwt::claims_for;
    #[cfg(feature = "auth")]
    pub use super::auth::jwt::VerifiedClaims;
    #[cfg(feature = "auth")]
    pub use super::auth::jwt::JWT;
    #[cfg(feature = "auth")]
    pub use super::auth::AuthorizedBearer;
    #[cfg(feature = "auth")]
    pub use super::auth::AuthorizedCookie;
    #[cfg(feature = "auth")]
    pub use super::auth::BearerToken;
    #[cfg(feature = "auth")]
    pub use super::auth::CookieToken;
    #[cfg(feature = "auth")]
    pub use axum_extra::extract::CookieJar;
    #[cfg(feature = "auth")]
    pub use jsonwebtoken::DecodingKey;
}
