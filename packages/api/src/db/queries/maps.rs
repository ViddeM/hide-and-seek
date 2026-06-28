#[cfg(feature = "server")]
use sqlx::{PgPool, FromRow};
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use crate::types::map_size::MapSize;

#[cfg(feature = "server")]
#[derive(Debug, Clone, FromRow)]
pub struct MapSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
}

#[cfg(feature = "server")]
/// Fetch all maps from the database
pub async fn get_all_maps(pool: &PgPool) -> Result<Vec<MapSummaryRow>, sqlx::Error> {
    sqlx::query_as::<_, MapSummaryRow>("SELECT id, name, size FROM maps ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}
