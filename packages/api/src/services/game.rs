use sqlx::PgPool;
use uuid::Uuid;

use crate::{db::queries, error::AppError, types::game_code::GameCode};

pub struct CreatedGame {
    pub game_code: GameCode,
    pub game_id: Uuid,
}

pub async fn create_game(pool: &PgPool, map_id: Uuid) -> Result<CreatedGame, AppError> {
    let code = GameCode::random();
    let game_id = queries::game::insert_game(pool, &code, map_id)
        .await
        .map_err(AppError::from)?;

    Ok(CreatedGame { game_code: code, game_id })
}
