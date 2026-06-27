use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::types::Coordinate;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Circle {
    pub id: Uuid,
    pub center_lat: Coordinate,
    pub center_lng: Coordinate,
    pub radius_meters: i64,
}
