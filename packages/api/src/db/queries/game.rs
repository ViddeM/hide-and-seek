use crate::db::tables::game::Game;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_game_by_id(pool: &PgPool, game_id: Uuid) -> Result<Game, sqlx::Error> {
    sqlx::query_as::<_, Game>(
        //language=PostgreSQL
        "SELECT id, code, map_id, status, created_at, started_at, finished_at FROM games WHERE id = $1",
    )
    .bind(game_id)
    .fetch_one(pool)
    .await
}

pub async fn insert_game(pool: &PgPool, code: &str, map_id: Uuid) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        //language=PostgreSQL
        r#"
        INSERT INTO games (code, map_id)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(code)
    .bind(map_id)
    .fetch_one(pool)
    .await
}
