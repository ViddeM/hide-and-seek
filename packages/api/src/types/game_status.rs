use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(type_name = "game_status", rename_all = "snake_case"))]
pub enum GameStatus {
    Lobby,
    Active,
    Finished,
}

impl std::fmt::Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::Lobby => write!(f, "Lobby"),
            GameStatus::Active => write!(f, "Active"),
            GameStatus::Finished => write!(f, "Finished"),
        }
    }
}
