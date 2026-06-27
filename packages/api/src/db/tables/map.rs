use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::db::tables::map_size::MapSize;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Map {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
    pub bounds: Uuid,
    pub created_at: DateTime<Utc>,
}
