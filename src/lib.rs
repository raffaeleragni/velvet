mod app;
mod auth;
mod client;
mod db;
mod errors;

#[macro_use]
pub mod prelude {
    pub use super::app::App;
    pub use super::auth::claims_for;
    pub use super::auth::AuthorizedBearer;
    pub use super::auth::AuthorizedCookie;
    pub use super::auth::BearerToken;
    pub use super::auth::CookieToken;
    pub use super::auth::VerifiedClaims;
    pub use super::auth::JWT;
    pub use super::client::client;
    pub use super::db::database;
    pub use super::errors::AppError;
    pub use super::errors::AppResult;
    pub use askama::Template;
    pub use axum::extract::{Form, Json, Path};
    pub use axum::http::StatusCode;
    pub use axum::response::Redirect;
    pub use axum::routing::{delete, get, patch, post, put};
    pub use axum::{Extension, Router};
    pub use axum_extra::extract::CookieJar;
    pub use axum_test::{TestServer, TestServerConfig};
    pub use jsonwebtoken::DecodingKey;
    pub use reqwest::Client;
    pub use rust_embed::RustEmbed;
    pub use serde::{Deserialize, Serialize};
    pub use sqlx::{query, query_as, Pool, Postgres};
    pub use tracing::{debug, error, info, instrument, span, trace, warn, Level};
    pub use valuable::Valuable;
}
