#[cfg(feature = "server")]
use axum::Extension;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::area::Area;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExclusionZoneResponse {
    pub id: Uuid,
    pub exclude_outside: bool,
    pub label: String,
    pub area: Area,
}

#[get("/api/games/{game_id}/exclusion_zones", pool: Extension<PgPool>)]
pub async fn list_game_exclusion_zones(game_id: Uuid) -> Result<Vec<ExclusionZoneResponse>> {
    todo!()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddZoneRequest {
    label: String,
    exclude_outside: bool,
    area: Area,
}

#[post("/api/games/{game_id}/exclusion_zones", pool: Extension<PgPool>)]
pub async fn create_exclusion_zone(
    game_id: Uuid,
    request: AddZoneRequest,
) -> Result<ExclusionZoneResponse> {
    todo!()
}

#[delete("/api/games/{game_id}/exclusion_zones/{zone_id}", pool: Extension<PgPool>)]
pub async fn remove_exclusion_zone(game_id: Uuid, zone_id: Uuid) -> Result<()> {
    todo!()
}
