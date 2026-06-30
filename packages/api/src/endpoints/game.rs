use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::game_status::GameStatus;

#[cfg(feature = "server")]
use {crate::services::game as game_service, axum::Extension, sqlx::PgPool};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub map_id: Uuid,
    pub host_display_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGameResponse {
    pub game_code: String,
    pub game_id: Uuid,
}

#[post("/api/games", pool: Extension<PgPool>)]
pub async fn create_game(request: CreateGameRequest) -> Result<CreateGameResponse> {
    let result = game_service::create_game(&pool, request.map_id).await?;

    Ok(CreateGameResponse {
        game_code: result.game_code.to_string(),
        game_id: result.game_id,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameResponse {
    pub game_id: Uuid,
    pub game_code: String,
    pub map_id: Uuid,
    pub status: GameStatus,
}

#[get("/api/games/{game_id}", pool: Extension<PgPool>)]
pub async fn get_game(game_id: Uuid) -> Result<GameResponse> {
    let game = game_service::get_game(&pool, game_id).await?;
    Ok(GameResponse {
        game_id: game.game_id,
        game_code: game.game_code.to_string(),
        map_id: game.map_id,
        status: game.status,
    })
}
