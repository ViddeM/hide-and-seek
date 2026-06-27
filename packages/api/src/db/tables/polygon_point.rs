use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::types::Coordinate;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct PolygonPoint {
    pub id: Uuid,
    pub polygon_id: Uuid,
    pub lat: Coordinate,
    pub lng: Coordinate,
}
