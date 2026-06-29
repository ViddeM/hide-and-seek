use api::types::map_size::MapSize;
use sqlx::PgPool;
use uuid::Uuid;

pub struct SeededMap {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
}

/// Seed a polygon + map with default values. Returns the map id.
pub async fn seed_map(pool: &PgPool) -> Uuid {
    seed_named_map(pool, "Test Map", MapSize::Small).await.id
}

/// Seed a polygon + named map. Returns a `SeededMap` with the id and the
/// values that were written, so callers can assert against them.
pub async fn seed_named_map(pool: &PgPool, name: &str, size: MapSize) -> SeededMap {
    let polygon_id: Uuid = sqlx::query_scalar("INSERT INTO polygon DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .expect("Failed to seed polygon");

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO maps (name, size, bounds) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(name)
    .bind(size)
    .bind(polygon_id)
    .fetch_one(pool)
    .await
    .expect("Failed to seed map");

    SeededMap {
        id,
        name: name.to_string(),
        size,
    }
}
