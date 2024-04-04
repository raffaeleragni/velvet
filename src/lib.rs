mod app;
mod client;
mod cookies;
mod db;
mod errors;

#[macro_use]
pub mod prelude {
    pub use super::app::App;
    pub use super::client::client;
    pub use super::cookies::CookieToken;
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
    pub use reqwest::Client;
    pub use rust_embed::RustEmbed;
    pub use serde::{Deserialize, Serialize};
    pub use sqlx::{query, query_as, Pool, Postgres};
    pub use tracing::{debug, error, info, instrument, span, trace, warn, Level};
    pub use valuable::Valuable;
}
