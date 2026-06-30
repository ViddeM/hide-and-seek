use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::types::{game_code::GameCode, game_status::GameStatus};

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Game {
    pub id: Uuid,
    pub code: GameCode,
    pub map_id: Uuid,
    pub status: GameStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}
