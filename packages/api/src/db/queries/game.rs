use sqlx::PgPool;
use uuid::Uuid;

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
