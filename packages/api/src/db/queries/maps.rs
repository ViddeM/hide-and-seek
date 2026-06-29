use crate::db::tables::map::Map;
use sqlx::PgPool;

/// Fetch all maps from the database
pub async fn get_all_maps(pool: &PgPool) -> Result<Vec<Map>, sqlx::Error> {
    sqlx::query_as::<_, Map>(
        //language=PostgreSQL
        r#"
        SELECT id, name, size, bounds, created_at 
        FROM maps 
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}
