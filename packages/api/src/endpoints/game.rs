use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "server")]
use {axum::Extension, crate::services::game as game_service, sqlx::PgPool};

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
