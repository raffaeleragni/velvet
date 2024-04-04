use askama_axum::IntoResponse;
use reqwest::StatusCode;
use tracing::error;

pub type AppResult<T> = Result<T, AppError>;

pub struct AppError(anyhow::Error);

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self(value.into())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
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
