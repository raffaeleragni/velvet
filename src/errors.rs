use std::{env::VarError, io};

use axum::{
    body::Body,
    http::Response,
    response::{IntoResponse, Redirect},
};
use reqwest::StatusCode;
use tracing::error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    error: anyhow::Error,
    redirect: Option<Redirect>,
}

impl From<Redirect> for AppError {
    fn from(redirect: Redirect) -> Self {
        Self {
            status: StatusCode::PERMANENT_REDIRECT,
            error: anyhow::anyhow!("None"),
            redirect: Some(redirect),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: value.into(),
            redirect: None,
        }
    }
}

#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: value.into(),
            redirect: None,
        }
    }
}

#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: value.into(),
            redirect: None,
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: value.into(),
            redirect: None,
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: value,
            redirect: None,
        }
    }
}

impl From<VarError> for AppError {
    fn from(value: VarError) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: value.into(),
            redirect: None,
        }
    }
}

impl From<StatusCode> for AppError {
    fn from(status: StatusCode) -> Self {
        Self {
            status,
            error: anyhow::Error::msg(status.canonical_reason().unwrap_or("")),
            redirect: None,
        }
    }
}

#[cfg(feature = "login")]
impl From<argon2::password_hash::Error> for AppError {
    fn from(error: argon2::password_hash::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: error.into(),
            redirect: None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        if let Some(r) = self.redirect {
            return r.into_response();
        }
        error!("Status: {}, Error: {}", self.status, self.error);
        (self.status, "Error").into_response()
    }
}
