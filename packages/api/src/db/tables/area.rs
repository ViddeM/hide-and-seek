use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Area {
    pub id: Uuid,
    pub line_id: Option<Uuid>,
    pub circle_id: Option<Uuid>,
    pub polygon_id: Option<Uuid>,
}
