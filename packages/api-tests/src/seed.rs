use api::types::{Point, area::{Area, Circle, Polygon}, game_code::GameCode, map_size::MapSize};
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

pub struct SeededGame {
    pub id: Uuid,
    pub code: GameCode,
    pub map_id: Uuid,
}

/// Seed a game row with a given map_id. The game starts in the 'lobby' status.
pub async fn seed_game(pool: &PgPool, map_id: Uuid) -> SeededGame {
    let code = GameCode::random();
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO games (code, map_id) VALUES ($1, $2) RETURNING id",
    )
    .bind(code.to_string())
    .bind(map_id)
    .fetch_one(pool)
    .await
    .expect("Failed to seed game");

    SeededGame { id, code, map_id }
}

pub struct SeededExclusionZone {
    pub id: Uuid,
    pub exclude_outside: bool,
    pub label: Option<String>,
    pub area: Area,
}

pub async fn seed_exclusion_zone_circle(pool: &PgPool, game_id: Uuid) -> SeededExclusionZone {
    let center_lat = 59.330_f64;
    let center_lng = 18.065_f64;
    let radius: i32 = 200;

    let circle_id: Uuid = sqlx::query_scalar(
        "INSERT INTO circle (center_lat, center_lng, radius_meters) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(center_lat)
    .bind(center_lng)
    .bind(radius)
    .fetch_one(pool)
    .await
    .expect("Failed to seed circle");

    let area_id: Uuid = sqlx::query_scalar("INSERT INTO area (circle_id) VALUES ($1) RETURNING id")
        .bind(circle_id)
        .fetch_one(pool)
        .await
        .expect("Failed to seed area");

    let label = Some("Circle zone".to_string());
    let exclude_outside = false;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO exclusion_zones (game_id, area_id, exclude_outside, label) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(game_id)
    .bind(area_id)
    .bind(exclude_outside)
    .bind(&label)
    .fetch_one(pool)
    .await
    .expect("Failed to seed exclusion zone");

    SeededExclusionZone {
        id,
        exclude_outside,
        label,
        area: Area::Circle(Circle {
            center: Point { lat: center_lat, lng: center_lng },
            radius: radius as f64,
        }),
    }
}

pub async fn seed_exclusion_zone_polygon(pool: &PgPool, game_id: Uuid) -> SeededExclusionZone {
    let vertices = vec![
        Point { lat: 59.330, lng: 18.065 },
        Point { lat: 59.335, lng: 18.065 },
        Point { lat: 59.330, lng: 18.075 },
    ];

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

    let area_id: Uuid = sqlx::query_scalar("INSERT INTO area (polygon_id) VALUES ($1) RETURNING id")
        .bind(polygon_id)
        .fetch_one(pool)
        .await
        .expect("Failed to seed area");

    let label = Some("Polygon zone".to_string());
    let exclude_outside = true;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO exclusion_zones (game_id, area_id, exclude_outside, label) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(game_id)
    .bind(area_id)
    .bind(exclude_outside)
    .bind(&label)
    .fetch_one(pool)
    .await
    .expect("Failed to seed exclusion zone");

    SeededExclusionZone {
        id,
        exclude_outside,
        label,
        area: Area::Polygon(Polygon { vertices }),
    }
}
