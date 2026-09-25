use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{
    area::Polygon,
    map_size::MapSize,
    map_status::MapStatus,
    transit::{TransitData, TransitRoute},
};

#[cfg(feature = "server")]
use {
    crate::services, crate::services::maps as map_service, axum::extract::Extension, sqlx::PgPool,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapSummary {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
    pub status: MapStatus,
}

#[cfg(feature = "server")]
impl From<services::maps::MapSummary> for MapSummary {
    fn from(value: services::maps::MapSummary) -> Self {
        Self {
            id: value.id,
            name: value.name,
            size: value.size,
            status: value.status,
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
            status: value.status,
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
    pub transit_routes: Vec<TransitRoute>,
}

#[post("/api/maps", pool: Extension<PgPool>)]
pub async fn create_map(request: CreateMapRequest) -> Result<MapSummary> {
    let map = map_service::create_map(
        &pool,
        request.name,
        request.size,
        request.bounds,
        request.transit_routes,
    )
    .await?;
    Ok(map.into())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
    pub boundary: Polygon,
    pub status: MapStatus,
}

#[get("/api/maps/{map_id}", pool: Extension<PgPool>)]
pub async fn get_map(map_id: Uuid) -> Result<MapDetailResponse> {
    let map = map_service::get_map(&pool, map_id).await?;
    Ok(map.into())
}

#[get("/api/maps/{map_id}/transit", pool: Extension<PgPool>)]
pub async fn get_transit(map_id: Uuid) -> Result<TransitData> {
    let data = map_service::get_transit_data(&pool, map_id).await?;
    Ok(data)
}

// ── Draft / step-by-step creation ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDraftMapRequest {
    pub name: String,
    pub size: MapSize,
}

/// Step 1: create a draft map (name + size only, no boundary yet).
#[post("/api/maps/draft", pool: Extension<PgPool>)]
pub async fn create_draft_map(request: CreateDraftMapRequest) -> Result<MapSummary> {
    let map = map_service::create_draft_map(&pool, request.name, request.size).await?;
    Ok(map.into())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateMapInfoRequest {
    pub name: String,
    pub size: MapSize,
}

/// Edit the name/size of an existing map.
#[post("/api/maps/{map_id}/info", pool: Extension<PgPool>)]
pub async fn update_map_info(map_id: Uuid, request: UpdateMapInfoRequest) -> Result<()> {
    map_service::update_map_info(&pool, map_id, request.name, request.size).await?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetBoundaryRequest {
    pub boundary: Polygon,
}

/// Step 2: save or replace the map boundary.
#[post("/api/maps/{map_id}/boundary", pool: Extension<PgPool>)]
pub async fn set_map_boundary(map_id: Uuid, request: SetBoundaryRequest) -> Result<()> {
    map_service::set_map_boundary(&pool, map_id, request.boundary).await?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveTransitRequest {
    pub routes: Vec<TransitRoute>,
}

/// Step 3: replace all transit data for a map.
#[post("/api/maps/{map_id}/transit", pool: Extension<PgPool>)]
pub async fn save_map_transit(map_id: Uuid, request: SaveTransitRequest) -> Result<()> {
    map_service::save_map_transit(&pool, map_id, request.routes).await?;
    Ok(())
}

/// Step 4: mark a draft map as complete.
#[post("/api/maps/{map_id}/finalize", pool: Extension<PgPool>)]
pub async fn finalize_map(map_id: Uuid) -> Result<MapSummary> {
    let map = map_service::finalize_map(&pool, map_id).await?;
    Ok(map.into())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewTransitRequest {
    pub size: MapSize,
    pub bounds: Polygon,
}

#[post("/api/transit/preview")]
pub async fn preview_transit(req: PreviewTransitRequest) -> Result<TransitData> {
    log::info!("Transit fetch starting for {:?} map", req.size);
    let routes = crate::services::overpass::fetch_transit(req.size, &req.bounds)
        .await
        .map_err(|e| {
            log::error!("Transit fetch failed: {e:#}");
            dioxus::prelude::ServerFnError::new(
                "Could not fetch transit routes from OpenStreetMap. All mirrors are unavailable — please try again later.".to_string(),
            )
        })?;
    let n_routes = routes.len();
    let n_waypoints: usize = routes.iter().flat_map(|r| r.waypoints.iter()).map(|s| s.len()).sum();
    let n_stops: usize = routes.iter().map(|r| r.stops.len()).sum();
    log::info!("Transit fetch done: {n_routes} routes, {n_waypoints} total waypoints, {n_stops} total stops");
    Ok(TransitData { routes })
}
