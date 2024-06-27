use askama_axum::IntoResponse;
use reqwest::StatusCode;
use tracing::error;

pub type AppResult<T> = Result<T, AppError>;

pub struct AppError {
    status: StatusCode,
    error: anyhow::Error,
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: value.into(),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: value.into(),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: value,
        }
    }
}

impl From<StatusCode> for AppError {
    fn from(status: StatusCode) -> Self {
        Self {
            status,
            error: anyhow::Error::msg(status.canonical_reason().unwrap_or("")),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> askama_axum::Response {
        error!("Error: {}", self.error);
        (self.status, "Internal Server Error").into_response()
    }
}
