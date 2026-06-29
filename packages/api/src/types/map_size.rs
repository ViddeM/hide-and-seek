use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(
    feature = "server",
    sqlx(type_name = "map_size", rename_all = "snake_case")
)]
pub enum MapSize {
    Small,
    Medium,
    Large,
}

impl Display for MapSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size_str = match self {
            MapSize::Small => "Small",
            MapSize::Medium => "Medium",
            MapSize::Large => "Large",
        };
        write!(f, "{}", size_str)
    }
}
