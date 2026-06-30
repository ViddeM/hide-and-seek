#[cfg(feature = "server")]
use axum::Extension;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::area::Area;

#[cfg(feature = "server")]
use crate::services::exclusion_zones as exclusion_zone_service;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExclusionZoneResponse {
    pub id: Uuid,
    pub exclude_outside: bool,
    pub label: Option<String>,
    pub area: Area,
}

#[cfg(feature = "server")]
impl From<exclusion_zone_service::ExclusionZoneDetail> for ExclusionZoneResponse {
    fn from(value: exclusion_zone_service::ExclusionZoneDetail) -> Self {
        Self {
            id: value.id,
            exclude_outside: value.exclude_outside,
            label: value.label,
            area: value.area,
        }
    }
}

#[get("/api/games/{game_id}/exclusion_zones", pool: Extension<PgPool>)]
pub async fn list_game_exclusion_zones(game_id: Uuid) -> Result<Vec<ExclusionZoneResponse>> {
    let zones = exclusion_zone_service::list_exclusion_zones(&pool, game_id).await?;
    Ok(zones.into_iter().map(|z| z.into()).collect())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddZoneRequest {
    pub label: Option<String>,
    pub exclude_outside: bool,
    pub area: Area,
}

#[post("/api/games/{game_id}/exclusion_zones", pool: Extension<PgPool>)]
pub async fn create_exclusion_zone(
    game_id: Uuid,
    request: AddZoneRequest,
) -> Result<ExclusionZoneResponse> {
    let zone = exclusion_zone_service::create_exclusion_zone(
        &pool,
        game_id,
        request.exclude_outside,
        request.label,
        request.area,
    )
    .await?;
    Ok(zone.into())
}

#[delete("/api/games/{game_id}/exclusion_zones/{zone_id}", pool: Extension<PgPool>)]
pub async fn remove_exclusion_zone(game_id: Uuid, zone_id: Uuid) -> Result<()> {
    exclusion_zone_service::remove_exclusion_zone(&pool, game_id, zone_id).await?;
    Ok(())
}
