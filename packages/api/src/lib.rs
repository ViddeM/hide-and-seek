pub mod config;
pub mod db;
pub mod error;
pub mod models;

// Server-function modules — bodies are server-only, but types are shared
pub mod auth;
pub mod cards;
pub mod game;
pub mod maps;
pub mod zones;

// Server-only modules
#[cfg(feature = "server")]
pub mod jwt;
#[cfg(feature = "server")]
pub mod middleware;
pub mod ws;

pub use error::AppError;
pub use models::*;
