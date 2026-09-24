use sqlx::PgPool;
use uuid::Uuid;

use crate::db::queries;
use crate::error::AppError;
use crate::types::{
    area::Polygon,
    map_size::MapSize,
    map_status::MapStatus,
    transit::{TransitData, TransitRoute},
};

#[derive(Debug, Clone, PartialEq)]
pub struct MapDetail {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
    pub boundary: Polygon,
    pub status: MapStatus,
}

pub async fn get_map(pool: &PgPool, map_id: Uuid) -> Result<MapDetail, AppError> {
    let map = queries::maps::get_map_by_id(pool, map_id)
        .await
        .map_err(AppError::from)?;
    let vertices = match map.bounds {
        Some(polygon_id) => queries::maps::get_polygon_points(pool, polygon_id)
            .await
            .map_err(AppError::from)?,
        None => vec![],
    };
    Ok(MapDetail {
        id: map.id,
        name: map.name,
        size: map.size,
        boundary: Polygon { vertices },
        status: map.status,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapSummary {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
    pub status: MapStatus,
}

pub async fn create_draft_map(pool: &PgPool, name: String, size: MapSize) -> Result<MapSummary, AppError> {
    let id = queries::maps::insert_draft_map(pool, &name, size)
        .await
        .map_err(AppError::from)?;
    Ok(MapSummary { id, name, size, status: MapStatus::Draft })
}

pub async fn update_map_info(pool: &PgPool, map_id: Uuid, name: String, size: MapSize) -> Result<(), AppError> {
    queries::maps::update_map_info(pool, map_id, &name, size)
        .await
        .map_err(AppError::from)
}

pub async fn set_map_boundary(pool: &PgPool, map_id: Uuid, boundary: Polygon) -> Result<(), AppError> {
    queries::maps::set_map_boundary(pool, map_id, &boundary.vertices)
        .await
        .map_err(AppError::from)
}

pub async fn save_map_transit(pool: &PgPool, map_id: Uuid, routes: Vec<TransitRoute>) -> Result<(), AppError> {
    queries::transit::delete_transit_for_map(pool, map_id)
        .await
        .map_err(AppError::from)?;
    if !routes.is_empty() {
        queries::transit::insert_transit_data(pool, map_id, &routes)
            .await
            .map_err(AppError::from)?;
    }
    Ok(())
}

pub async fn finalize_map(pool: &PgPool, map_id: Uuid) -> Result<MapSummary, AppError> {
    queries::maps::set_map_status(pool, map_id, MapStatus::Complete)
        .await
        .map_err(AppError::from)?;
    let map = queries::maps::get_map_by_id(pool, map_id)
        .await
        .map_err(AppError::from)?;
    Ok(MapSummary { id: map.id, name: map.name, size: map.size, status: MapStatus::Complete })
}

pub async fn create_map(
    pool: &PgPool,
    name: String,
    size: MapSize,
    bounds: Polygon,
    transit_routes: Vec<TransitRoute>,
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

    if !transit_routes.is_empty() {
        if let Err(e) = queries::transit::insert_transit_data(pool, map_id, &transit_routes).await {
            tracing::warn!("Failed to store transit data for map {map_id}: {e}");
        }
    }

    Ok(MapSummary { id: map_id, name, size, status: MapStatus::Complete })
}

pub async fn get_transit_data(pool: &PgPool, map_id: Uuid) -> Result<TransitData, AppError> {
    queries::transit::get_transit_data(pool, map_id)
        .await
        .map_err(AppError::from)
}

pub async fn list_all_maps(pool: &PgPool) -> Result<Vec<MapSummary>, AppError> {
    let rows = queries::maps::get_all_maps(pool)
        .await
        .map_err(AppError::from)?;

    Ok(rows
        .into_iter()
        .map(|row| MapSummary {
            id: row.id,
            name: row.name,
            size: row.size,
            status: row.status,
        })
        .collect())
}
