use crate::types::Point;
use sqlx::{FromRow, PgConnection, PgPool};
use uuid::Uuid;

#[derive(FromRow)]
pub struct ExclusionZoneRow {
    pub id: Uuid,
    pub area_id: Uuid,
    pub exclude_outside: bool,
    pub label: Option<String>,
    pub line_id: Option<Uuid>,
    pub circle_id: Option<Uuid>,
    pub polygon_id: Option<Uuid>,
    pub start_lat: Option<f64>,
    pub start_lng: Option<f64>,
    pub end_lat: Option<f64>,
    pub end_lng: Option<f64>,
    pub center_lat: Option<f64>,
    pub center_lng: Option<f64>,
    pub radius_meters: Option<i64>,
}

pub async fn get_zones_for_game(
    pool: &PgPool,
    game_id: Uuid,
) -> Result<Vec<ExclusionZoneRow>, sqlx::Error> {
    sqlx::query_as::<_, ExclusionZoneRow>(
        //language=PostgreSQL
        r#"
        SELECT
            ez.id, ez.area_id, ez.exclude_outside, ez.label,
            a.line_id, a.circle_id, a.polygon_id,
            l.start_lat, l.start_lng, l.end_lat, l.end_lng,
            c.center_lat, c.center_lng, c.radius_meters
        FROM exclusion_zones ez
        JOIN area a ON a.id = ez.area_id
        LEFT JOIN line l ON l.id = a.line_id
        LEFT JOIN circle c ON c.id = a.circle_id
        WHERE ez.game_id = $1
        ORDER BY ez.created_at
        "#,
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
}

pub async fn get_zone_for_game(
    pool: &PgPool,
    game_id: Uuid,
    zone_id: Uuid,
) -> Result<ExclusionZoneRow, sqlx::Error> {
    sqlx::query_as::<_, ExclusionZoneRow>(
        //language=PostgreSQL
        r#"
        SELECT
            ez.id, ez.area_id, ez.exclude_outside, ez.label,
            a.line_id, a.circle_id, a.polygon_id,
            l.start_lat, l.start_lng, l.end_lat, l.end_lng,
            c.center_lat, c.center_lng, c.radius_meters
        FROM exclusion_zones ez
        JOIN area a ON a.id = ez.area_id
        LEFT JOIN line l ON l.id = a.line_id
        LEFT JOIN circle c ON c.id = a.circle_id
        WHERE ez.id = $1 AND ez.game_id = $2
        "#,
    )
    .bind(zone_id)
    .bind(game_id)
    .fetch_one(pool)
    .await
}

pub async fn insert_line(
    conn: &mut PgConnection,
    start_lat: f64,
    start_lng: f64,
    end_lat: f64,
    end_lng: f64,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        //language=PostgreSQL
        "INSERT INTO line (start_lat, start_lng, end_lat, end_lng) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(start_lat)
    .bind(start_lng)
    .bind(end_lat)
    .bind(end_lng)
    .fetch_one(conn)
    .await
}

pub async fn insert_circle(
    conn: &mut PgConnection,
    center_lat: f64,
    center_lng: f64,
    radius_meters: i32,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        //language=PostgreSQL
        "INSERT INTO circle (center_lat, center_lng, radius_meters) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(center_lat)
    .bind(center_lng)
    .bind(radius_meters)
    .fetch_one(conn)
    .await
}

pub async fn insert_area_for_line(
    conn: &mut PgConnection,
    line_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        //language=PostgreSQL
        "INSERT INTO area (line_id) VALUES ($1) RETURNING id",
    )
    .bind(line_id)
    .fetch_one(conn)
    .await
}

pub async fn insert_area_for_circle(
    conn: &mut PgConnection,
    circle_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        //language=PostgreSQL
        "INSERT INTO area (circle_id) VALUES ($1) RETURNING id",
    )
    .bind(circle_id)
    .fetch_one(conn)
    .await
}

pub async fn insert_area_for_polygon(
    conn: &mut PgConnection,
    polygon_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        //language=PostgreSQL
        "INSERT INTO area (polygon_id) VALUES ($1) RETURNING id",
    )
    .bind(polygon_id)
    .fetch_one(conn)
    .await
}

pub async fn insert_exclusion_zone(
    conn: &mut PgConnection,
    game_id: Uuid,
    area_id: Uuid,
    exclude_outside: bool,
    label: &Option<String>,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        //language=PostgreSQL
        "INSERT INTO exclusion_zones (game_id, area_id, exclude_outside, label) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(game_id)
    .bind(area_id)
    .bind(exclude_outside)
    .bind(label)
    .fetch_one(conn)
    .await
}

pub async fn delete_exclusion_zone(
    conn: &mut PgConnection,
    zone_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM exclusion_zones WHERE id = $1")
        .bind(zone_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

pub async fn delete_area(conn: &mut PgConnection, area_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM area WHERE id = $1")
        .bind(area_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

pub async fn delete_line(conn: &mut PgConnection, line_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM line WHERE id = $1")
        .bind(line_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

pub async fn delete_circle(conn: &mut PgConnection, circle_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM circle WHERE id = $1")
        .bind(circle_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

pub async fn delete_polygon(conn: &mut PgConnection, polygon_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM polygon_point WHERE polygon_id = $1")
        .bind(polygon_id)
        .execute(&mut *conn)
        .await?;
    sqlx::query("DELETE FROM polygon WHERE id = $1")
        .bind(polygon_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

pub async fn get_polygon_points(
    pool: &PgPool,
    polygon_id: Uuid,
) -> Result<Vec<Point>, sqlx::Error> {
    super::maps::get_polygon_points(pool, polygon_id).await
}
