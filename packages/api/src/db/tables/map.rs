use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::types::{map_size::MapSize, map_status::MapStatus};

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Map {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
    pub bounds: Option<Uuid>,
    pub status: MapStatus,
    pub created_at: DateTime<Utc>,
}
