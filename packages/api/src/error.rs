use dioxus::prelude::ServerFnError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[cfg(feature = "server")]
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::Internal("Not found".to_string()),
            _ => AppError::Database(e.to_string()),
        }
    }
}

impl From<AppError> for ServerFnError {
    fn from(e: AppError) -> Self {
        ServerFnError::new(e.to_string())
    }
}

#[cfg(feature = "server")]
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Database(msg) => {
                log::error!("Database error: {}", msg);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            AppError::Internal(msg) => {
                log::error!("Internal error: {}", msg);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };
        (status, message).into_response()
    }
}
