#[macro_use]
pub mod prelude {
    pub use super::database;
    pub use super::App;
    pub use super::AppError;
    pub use super::AppResult;
    pub use askama::Template;
    pub use axum::routing::{delete, get, patch, post, put};
    pub use axum::extract::{Path, Json, Form};
    pub use axum::{Extension, Router};
    pub use rust_embed::RustEmbed;
    pub use serde::{Deserialize, Serialize};
    pub use sqlx::{Pool, Postgres};
    pub use tracing::{debug, error, info, trace, warn};
}

use askama_axum::IntoResponse;
use axum::http::StatusCode;
use axum::Extension;
use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;
use rust_embed::RustEmbed;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use structured_logger::async_json::new_writer;
use tokio::net::TcpListener;
use tracing::error;
use tracing::info;

pub type AppResult<T> = anyhow::Result<T>;

pub struct AppError(anyhow::Error);

pub struct App {
    router: Router,
}

impl App {
    pub fn new(router: Router) -> Self {
        dotenvy::dotenv().ok();

        Self { router }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        start(self.router).await
    }

    pub fn inject<T: Clone + Send + Sync + 'static>(self, t: T) -> Self {
        Self {
            router: self.router.layer(Extension(t)),
        }
    }

    pub fn include_static<T: RustEmbed>(
        self,
        mime_type: &'static str,
        path: &'static str,
        file: &'static str,
    ) -> Self {
        Self {
            router: self.router.route(
                path,
                get(|| async {
                    (
                        [("Content-Type", mime_type.to_string())],
                        T::get(file).unwrap().data.to_vec(),
                    )
                        .into_response()
                }),
            ),
        }
    }
}

async fn start(app: Router) -> anyhow::Result<()> {
    let bind = env::var("SERVER_BIND").unwrap_or("0.0.0.0".into());
    let port = env::var("SERVER_PORT")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(8080);
    let listener = TcpListener::bind(format!("{bind}:{port}")).await?;

    logger();
    let _guard = sentry();

    let app = prometheus(app);
    let app = app.route("/status/liveness", get(|| async { "".into_response() }));

    info!("Starting server on {bind}:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn sentry() -> Option<sentry::ClientInitGuard> {
    if let Ok(url) = env::var("SENTRY_URL") {
        return Some(sentry::init((
            url,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: 1.0,
                ..Default::default()
            },
        )));
    }
    None
}

fn logger() {
    let enabled: bool = env::var("STRUCTURED_LOGGING")
        .map(|s| s.parse::<bool>().unwrap_or(false))
        .unwrap_or(false);
    if enabled {
        structured_logger::Builder::with_level("info")
            .with_target_writer("*", new_writer(tokio::io::stdout()))
            .init();
    } else {
        tracing_subscriber::fmt::init();
    }
}

fn prometheus(app: Router) -> Router {
    let (metric_gatherer, metric_printer) = PrometheusMetricLayer::pair();
    app.route(
        "/metrics/prometheus",
        get(|| async move { metric_printer.render() }),
    )
    .layer(metric_gatherer)
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self(value.into())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> askama_axum::Response {
        error!("Error: {}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
    }
}

pub async fn database() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");
    let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .unwrap()
}
