use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct ExclusionZone {
    id: Uuid,
    game_id: Uuid,
    area_id: Uuid,
    exclude_outside: bool,
    label: Option<String>,
    created_at: DateTime<Utc>,
}
