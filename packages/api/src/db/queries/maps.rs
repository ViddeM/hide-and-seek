use crate::db::tables::map::Map;
use crate::types::{map_size::MapSize, map_status::MapStatus, Point};
use sqlx::{FromRow, PgConnection, PgPool};
use uuid::Uuid;

#[derive(FromRow)]
struct PointRow {
    lat: f64,
    lng: f64,
}

pub async fn get_all_maps(pool: &PgPool) -> Result<Vec<Map>, sqlx::Error> {
    sqlx::query_as::<_, Map>(
        "SELECT id, name, size, bounds, status, created_at FROM maps ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_map_by_id(pool: &PgPool, map_id: Uuid) -> Result<Map, sqlx::Error> {
    sqlx::query_as::<_, Map>(
        "SELECT id, name, size, bounds, status, created_at FROM maps WHERE id = $1",
    )
    .bind(map_id)
    .fetch_one(pool)
    .await
}

pub async fn get_polygon_points(pool: &PgPool, polygon_id: Uuid) -> Result<Vec<Point>, sqlx::Error> {
    sqlx::query_as::<_, PointRow>(
        "SELECT lat, lng FROM polygon_point WHERE polygon_id = $1 ORDER BY number",
    )
    .bind(polygon_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| Point { lat: r.lat, lng: r.lng }).collect())
}

pub async fn insert_polygon(conn: &mut PgConnection) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO polygon DEFAULT VALUES RETURNING id")
        .fetch_one(conn)
        .await
}

pub async fn insert_polygon_points(
    conn: &mut PgConnection,
    polygon_id: Uuid,
    vertices: &[Point],
) -> Result<(), sqlx::Error> {
    for (number, vertex) in vertices.iter().enumerate() {
        sqlx::query(
            "INSERT INTO polygon_point (number, polygon_id, lat, lng) VALUES ($1, $2, $3, $4)",
        )
        .bind(number as i32)
        .bind(polygon_id)
        .bind(vertex.lat)
        .bind(vertex.lng)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

pub async fn insert_map(
    conn: &mut PgConnection,
    name: &str,
    size: MapSize,
    polygon_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO maps (name, size, bounds, status) VALUES ($1, $2, $3, 'complete') RETURNING id",
    )
    .bind(name)
    .bind(size)
    .bind(polygon_id)
    .fetch_one(conn)
    .await
}

/// Create a draft map with no boundary. Returns the new map id.
pub async fn insert_draft_map(pool: &PgPool, name: &str, size: MapSize) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO maps (name, size, status) VALUES ($1, $2, 'draft') RETURNING id",
    )
    .bind(name)
    .bind(size)
    .fetch_one(pool)
    .await
}

/// Update the name and size of an existing map.
pub async fn update_map_info(pool: &PgPool, map_id: Uuid, name: &str, size: MapSize) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE maps SET name = $1, size = $2 WHERE id = $3")
        .bind(name)
        .bind(size)
        .bind(map_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Replace the boundary of a map. Creates a new polygon if needed, replaces old one.
pub async fn set_map_boundary(pool: &PgPool, map_id: Uuid, vertices: &[Point]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Get old bounds polygon id (if any)
    let old_bounds: Option<Uuid> = sqlx::query_scalar("SELECT bounds FROM maps WHERE id = $1")
        .bind(map_id)
        .fetch_one(&mut *tx)
        .await?;

    // Create new polygon and insert points
    let new_polygon_id: Uuid = sqlx::query_scalar("INSERT INTO polygon DEFAULT VALUES RETURNING id")
        .fetch_one(&mut *tx)
        .await?;

    for (number, vertex) in vertices.iter().enumerate() {
        sqlx::query(
            "INSERT INTO polygon_point (number, polygon_id, lat, lng) VALUES ($1, $2, $3, $4)",
        )
        .bind(number as i32)
        .bind(new_polygon_id)
        .bind(vertex.lat)
        .bind(vertex.lng)
        .execute(&mut *tx)
        .await?;
    }

    // Point map at new polygon
    sqlx::query("UPDATE maps SET bounds = $1 WHERE id = $2")
        .bind(new_polygon_id)
        .bind(map_id)
        .execute(&mut *tx)
        .await?;

    // Clean up old polygon (polygon_point has no cascade, so delete manually)
    if let Some(old_id) = old_bounds {
        sqlx::query("DELETE FROM polygon_point WHERE polygon_id = $1")
            .bind(old_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM polygon WHERE id = $1")
            .bind(old_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await
}

/// Mark a map as complete (draft → complete).
pub async fn set_map_status(pool: &PgPool, map_id: Uuid, status: MapStatus) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE maps SET status = $1 WHERE id = $2")
        .bind(status)
        .bind(map_id)
        .execute(pool)
        .await?;
    Ok(())
}
