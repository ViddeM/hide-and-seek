use sqlx::PgPool;
use uuid::Uuid;

use crate::db::queries;
use crate::error::AppError;
use crate::types::{area::Polygon, map_size::MapSize};

#[derive(Debug, Clone, PartialEq)]
pub struct MapDetail {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
    pub boundary: Polygon,
}

pub async fn get_map(pool: &PgPool, map_id: Uuid) -> Result<MapDetail, AppError> {
    let map = queries::maps::get_map_by_id(pool, map_id)
        .await
        .map_err(AppError::from)?;
    let points = queries::maps::get_polygon_points(pool, map.bounds)
        .await
        .map_err(AppError::from)?;
    Ok(MapDetail {
        id: map.id,
        name: map.name,
        size: map.size,
        boundary: Polygon { vertices: points },
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapSummary {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
}

pub async fn create_map(
    pool: &PgPool,
    name: String,
    size: MapSize,
    bounds: Polygon,
) -> Result<MapSummary, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;

    let polygon_id = queries::maps::insert_polygon(&mut *tx)
        .await
        .map_err(AppError::from)?;

    queries::maps::insert_polygon_points(&mut *tx, polygon_id, &bounds.vertices)
        .await
        .map_err(AppError::from)?;

    let map_id = queries::maps::insert_map(&mut *tx, &name, size, polygon_id)
        .await
        .map_err(AppError::from)?;

    tx.commit().await.map_err(AppError::from)?;

    Ok(MapSummary { id: map_id, name, size })
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
