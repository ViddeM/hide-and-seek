use api::types::{Point, map_size::MapSize};
use sqlx::PgPool;
use uuid::Uuid;

pub struct SeededMap {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
    pub vertices: Vec<Point>,
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
        vertices: vec![],
    }
}

/// Seed a polygon with points, then a named map. Returns a `SeededMap` including
/// the vertices so callers can assert the boundary round-trips correctly.
pub async fn seed_map_with_boundary(pool: &PgPool, name: &str, size: MapSize, vertices: Vec<Point>) -> SeededMap {
    let polygon_id: Uuid = sqlx::query_scalar("INSERT INTO polygon DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .expect("Failed to seed polygon");

    for (number, vertex) in vertices.iter().enumerate() {
        sqlx::query(
            "INSERT INTO polygon_point (number, polygon_id, lat, lng) VALUES ($1, $2, $3, $4)",
        )
        .bind(number as i32)
        .bind(polygon_id)
        .bind(vertex.lat)
        .bind(vertex.lng)
        .execute(pool)
        .await
        .expect("Failed to seed polygon point");
    }

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO maps (name, size, bounds) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(name)
    .bind(size)
    .bind(polygon_id)
    .fetch_one(pool)
    .await
    .expect("Failed to seed map");

    SeededMap { id, name: name.to_string(), size, vertices }
}
