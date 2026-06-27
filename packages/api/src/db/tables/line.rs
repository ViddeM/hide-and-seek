use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::types::Coordinate;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Line {
    pub id: Uuid,
    pub start_lat: Coordinate,
    pub start_lng: Coordinate,
    pub end_lat: Coordinate,
    pub end_lng: Coordinate,
}
