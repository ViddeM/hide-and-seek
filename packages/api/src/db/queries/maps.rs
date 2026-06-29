use crate::db::tables::map::Map;
use crate::types::{map_size::MapSize, Point};
use sqlx::{FromRow, PgConnection, PgPool};
use uuid::Uuid;

#[derive(FromRow)]
struct PointRow {
    lat: f64,
    lng: f64,
}

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

pub async fn get_map_by_id(pool: &PgPool, map_id: Uuid) -> Result<Map, sqlx::Error> {
    sqlx::query_as::<_, Map>(
        //language=PostgreSQL
        "SELECT id, name, size, bounds, created_at FROM maps WHERE id = $1",
    )
    .bind(map_id)
    .fetch_one(pool)
    .await
}

pub async fn get_polygon_points(pool: &PgPool, polygon_id: Uuid) -> Result<Vec<Point>, sqlx::Error> {
    sqlx::query_as::<_, PointRow>(
        //language=PostgreSQL
        "SELECT lat, lng FROM polygon_point WHERE polygon_id = $1 ORDER BY number",
    )
    .bind(polygon_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| Point { lat: r.lat, lng: r.lng }).collect())
}

pub async fn insert_polygon(conn: &mut PgConnection) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        //language=PostgreSQL
        "INSERT INTO polygon DEFAULT VALUES RETURNING id",
    )
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
            //language=PostgreSQL
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
        //language=PostgreSQL
        "INSERT INTO maps (name, size, bounds) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(name)
    .bind(size)
    .bind(polygon_id)
    .fetch_one(conn)
    .await
}
