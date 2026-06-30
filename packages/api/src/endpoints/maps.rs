use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{area::Polygon, map_size::MapSize};

#[cfg(feature = "server")]
use {
    crate::services, crate::services::maps as map_service, axum::extract::Extension, sqlx::PgPool,
};

/// Response type for the list maps endpoint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapSummary {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
}

#[cfg(feature = "server")]
impl From<services::maps::MapSummary> for MapSummary {
    fn from(value: services::maps::MapSummary) -> Self {
        Self {
            id: value.id,
            name: value.name,
            size: value.size,
        }
    }
}

#[cfg(feature = "server")]
impl From<services::maps::MapDetail> for MapDetailResponse {
    fn from(value: services::maps::MapDetail) -> Self {
        Self {
            id: value.id,
            name: value.name,
            size: value.size,
            boundary: value.boundary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListMapsResponse {
    pub maps: Vec<MapSummary>,
}

#[get("/api/maps", pool: Extension<PgPool>)]
pub async fn list_maps() -> Result<ListMapsResponse> {
    let maps = map_service::list_all_maps(&pool).await?;

    Ok(ListMapsResponse {
        maps: maps.into_iter().map(|m| m.into()).collect(),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateMapRequest {
    pub name: String,
    pub size: MapSize,
    pub bounds: Polygon,
}

#[post("/api/maps", pool: Extension<PgPool>)]
pub async fn create_map(request: CreateMapRequest) -> Result<MapSummary> {
    let map = map_service::create_map(&pool, request.name, request.size, request.bounds).await?;

    Ok(map.into())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
    pub boundary: Polygon,
}

#[get("/api/maps/{map_id}", pool: Extension<PgPool>)]
pub async fn get_map(map_id: Uuid) -> Result<MapDetailResponse> {
    let map = map_service::get_map(&pool, map_id).await?;
    Ok(map.into())
}
