use crate::models::*;
use dioxus::prelude::*;
use uuid::Uuid;

/// List all available maps (no auth required).
#[server(endpoint = "/maps/list")]
pub async fn list_maps() -> Result<Vec<MapSummary>, ServerFnError> {
    use crate::AppError;
    use axum::extract::Extension;
    use sqlx::{PgPool, Row};

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;

    let rows = sqlx::query("SELECT id, name, size::text AS size FROM maps ORDER BY name")
        .fetch_all(&pool)
        .await
        .map_err(AppError::Database)?;

    let maps = rows
        .iter()
        .map(|r| MapSummary {
            id: r.try_get("id").unwrap_or_default(),
            name: r.try_get("name").unwrap_or_default(),
            size: parse_map_size(&r.try_get::<String, _>("size").unwrap_or_default()),
        })
        .collect();

    Ok(maps)
}

/// Get a map with all its stops and questions.
#[server(endpoint = "/maps/get")]
pub async fn get_map(map_id: Uuid) -> Result<MapDetail, ServerFnError> {
    use crate::AppError;
    use axum::extract::Extension;
    use sqlx::{PgPool, Row};

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;

    let map = sqlx::query(
        "SELECT id, name, size::text AS size, bounds_sw_lat, bounds_sw_lng, bounds_ne_lat, bounds_ne_lng
         FROM maps WHERE id = $1",
    )
    .bind(map_id)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("Map {map_id} not found")))?;

    let stops = sqlx::query(
        "SELECT id, name, lat, lng, stop_type FROM map_stops WHERE map_id = $1 ORDER BY name",
    )
    .bind(map_id)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?
    .iter()
    .map(|r| MapStop {
        id: r.try_get("id").unwrap_or_default(),
        name: r.try_get("name").unwrap_or_default(),
        lat: r.try_get("lat").unwrap_or_default(),
        lng: r.try_get("lng").unwrap_or_default(),
        stop_type: r.try_get("stop_type").unwrap_or_default(),
    })
    .collect();

    let questions = sqlx::query(
        "SELECT id, text, radius_m, requires_stop FROM map_questions WHERE map_id = $1 ORDER BY text",
    )
    .bind(map_id)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?
    .iter()
    .map(|r| MapQuestion {
        id: r.try_get("id").unwrap_or_default(),
        text: r.try_get("text").unwrap_or_default(),
        radius_m: r.try_get("radius_m").ok(),
        requires_stop: r.try_get("requires_stop").unwrap_or(false),
    })
    .collect();

    Ok(MapDetail {
        id: map.try_get("id").unwrap_or_default(),
        name: map.try_get("name").unwrap_or_default(),
        size: parse_map_size(&map.try_get::<String, _>("size").unwrap_or_default()),
        bounds: MapBounds {
            sw_lat: map.try_get("bounds_sw_lat").unwrap_or_default(),
            sw_lng: map.try_get("bounds_sw_lng").unwrap_or_default(),
            ne_lat: map.try_get("bounds_ne_lat").unwrap_or_default(),
            ne_lng: map.try_get("bounds_ne_lng").unwrap_or_default(),
        },
        stops,
        questions,
    })
}

/// Create a new map (no auth required — maps are shared resources created before a game exists).
#[server(endpoint = "/maps/create")]
pub async fn create_map(req: CreateMapRequest) -> Result<MapSummary, ServerFnError> {
    use crate::AppError;
    use axum::extract::Extension;
    use sqlx::PgPool;

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;

    let size_str = match req.size {
        MapSize::Small => "small",
        MapSize::Medium => "medium",
        MapSize::Large => "large",
    };

    let map_id: Uuid = sqlx::query_scalar(
        "INSERT INTO maps (name, size, bounds_sw_lat, bounds_sw_lng, bounds_ne_lat, bounds_ne_lng)
         VALUES ($1, $2::map_size, $3, $4, $5, $6) RETURNING id",
    )
    .bind(&req.name)
    .bind(size_str)
    .bind(req.bounds.sw_lat)
    .bind(req.bounds.sw_lng)
    .bind(req.bounds.ne_lat)
    .bind(req.bounds.ne_lng)
    .fetch_one(&pool)
    .await
    .map_err(AppError::Database)?;

    for stop in &req.stops {
        sqlx::query(
            "INSERT INTO map_stops (map_id, name, lat, lng, stop_type) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(map_id)
        .bind(&stop.name)
        .bind(stop.lat)
        .bind(stop.lng)
        .bind(&stop.stop_type)
        .execute(&pool)
        .await
        .map_err(AppError::Database)?;
    }

    for q in &req.questions {
        sqlx::query(
            "INSERT INTO map_questions (map_id, text, radius_m, requires_stop) VALUES ($1, $2, $3, $4)",
        )
        .bind(map_id)
        .bind(&q.text)
        .bind(q.radius_m)
        .bind(q.requires_stop)
        .execute(&pool)
        .await
        .map_err(AppError::Database)?;
    }

    log::info!("Map created: id={map_id} name={}", req.name);

    Ok(MapSummary { id: map_id, name: req.name, size: req.size })
}

#[cfg(feature = "server")]
fn parse_map_size(s: &str) -> MapSize {
    match s {
        "small" => MapSize::Small,
        "large" => MapSize::Large,
        _ => MapSize::Medium,
    }
}
