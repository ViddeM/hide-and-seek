#[cfg(feature = "server")]
mod inner {
    use clap::Parser;

    #[derive(Debug, Clone, Parser)]
    #[command(about = "Hide and Seek game server")]
    pub struct Config {
        /// PostgreSQL connection URL
        #[arg(long, env = "DATABASE_URL")]
        pub database_url: String,

        /// Secret key for signing JWTs (must be at least 32 characters)
        #[arg(long, env = "JWT_SECRET")]
        pub jwt_secret: String,

        /// Address and port to bind the HTTP server
        #[arg(long, env = "HOST", default_value = "0.0.0.0:8080")]
        pub host: String,

        /// Log filter string (e.g. "info,api=debug")
        #[arg(long, env = "RUST_LOG", default_value = "info")]
        pub rust_log: String,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ConfigError {
        #[error("JWT_SECRET must be at least 32 characters, got {0}")]
        JwtSecretTooShort(usize),
        #[error("DATABASE_URL does not look like a postgres:// URL")]
        InvalidDatabaseUrl,
        #[error("HOST is not a valid socket address: {0}")]
        InvalidHost(String),
    }

    impl Config {
        /// Load config from CLI args + environment (with .env support via dotenvy).
        /// Crashes on invalid configuration so the binary fails fast at startup.
        pub fn load() -> Self {
            dotenvy::dotenv().ok();
            let cfg = Self::parse();
            if let Err(e) = cfg.validate() {
                eprintln!("Configuration error: {e}");
                std::process::exit(1);
            }
            cfg
        }

        pub fn validate(&self) -> Result<(), ConfigError> {
            if self.jwt_secret.len() < 32 {
                return Err(ConfigError::JwtSecretTooShort(self.jwt_secret.len()));
            }
            if !self.database_url.starts_with("postgres://")
                && !self.database_url.starts_with("postgresql://")
            {
                return Err(ConfigError::InvalidDatabaseUrl);
            }
            self.host
                .parse::<std::net::SocketAddr>()
                .map_err(|e| ConfigError::InvalidHost(e.to_string()))?;
            Ok(())
        }

        pub fn socket_addr(&self) -> std::net::SocketAddr {
            self.host.parse().expect("validated in load()")
        }
    }
}

#[cfg(feature = "server")]
pub use inner::*;
