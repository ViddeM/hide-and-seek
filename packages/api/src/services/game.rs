use sqlx::PgPool;
use uuid::Uuid;

use crate::{db::queries, error::AppError, types::{game_code::GameCode, game_status::GameStatus}};

pub struct CreatedGame {
    pub game_code: GameCode,
    pub game_id: Uuid,
}

pub struct GameDetail {
    pub game_id: Uuid,
    pub game_code: GameCode,
    pub map_id: Uuid,
    pub status: GameStatus,
}

pub async fn get_game(pool: &PgPool, game_id: Uuid) -> Result<GameDetail, AppError> {
    let game = queries::game::get_game_by_id(pool, game_id)
        .await
        .map_err(AppError::from)?;
    Ok(GameDetail {
        game_id: game.id,
        game_code: game.code,
        map_id: game.map_id,
        status: game.status,
    })
}

pub async fn create_game(pool: &PgPool, map_id: Uuid) -> Result<CreatedGame, AppError> {
    let code = GameCode::random();
    let game_id = queries::game::insert_game(pool, &code, map_id)
        .await
        .map_err(AppError::from)?;

    Ok(CreatedGame { game_code: code, game_id })
}
