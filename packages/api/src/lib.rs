// Common
pub mod endpoints;
pub mod error;
pub mod types;

// Server only

#[cfg(feature = "server")]
pub mod cli;
#[cfg(feature = "server")]
pub mod db;
#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "server")]
pub mod services;

pub use error::AppError;
