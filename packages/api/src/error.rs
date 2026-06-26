use dioxus::prelude::ServerFnError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("authentication error: {0}")]
    Auth(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[cfg(feature = "server")]
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl From<AppError> for ServerFnError {
    fn from(e: AppError) -> Self {
        ServerFnError::new(e.to_string())
    }
}
