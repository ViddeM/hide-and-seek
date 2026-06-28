use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::map_size::MapSize;

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
