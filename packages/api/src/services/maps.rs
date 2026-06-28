use sqlx::PgPool;
use uuid::Uuid;

use crate::db::queries;
use crate::error::AppError;
use crate::types::map_size::MapSize;

#[derive(Debug, Clone, PartialEq)]
pub struct MapSummary {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
}

/// Fetch all available maps
pub async fn list_all_maps(pool: &PgPool) -> Result<Vec<MapSummary>, AppError> {
    let rows = queries::maps::get_all_maps(pool)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(rows
        .into_iter()
        .map(|row| MapSummary {
            id: row.id,
            name: row.name,
            size: row.size,
        })
        .collect())
}
