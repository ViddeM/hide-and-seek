#[cfg(feature = "server")]
mod inner {
    use sqlx::{postgres::PgPoolOptions, PgPool};
    use std::time::Duration;

    pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(pool)
    }
}

#[cfg(feature = "server")]
pub use inner::*;
