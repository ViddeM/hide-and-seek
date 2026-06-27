use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(sqlx::Type)]
#[sqlx(type_name = "map_size", rename_all = "snake_case")]
pub enum MapSize {
    Small,
    Medium,
    Large,
}
