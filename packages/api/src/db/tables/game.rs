use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{db::tables::game_status::GameStatus, types::game_code::GameCode};

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Game {
    id: Uuid,
    code: GameCode,
    map_id: Uuid,
    status: GameStatus,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}
