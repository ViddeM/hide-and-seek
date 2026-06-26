use crate::models::*;
use dioxus::prelude::*;
use uuid::Uuid;

/// Add an exclusion zone (seeker or host only). Broadcasts via WebSocket.
#[server(endpoint = "/zones/add")]
pub async fn add_exclusion_zone(
    game_id: Uuid,
    req: AddZoneRequest,
) -> Result<ExclusionZone, ServerFnError> {
    use crate::AppError;
    use axum::extract::Extension;
    use sqlx::{PgPool, Row};
    use std::sync::Arc;

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let config: Extension<Arc<crate::config::Config>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let hub: Extension<Arc<crate::ws::GameHub>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;
    let hub = hub.0;
    let claims = crate::auth::require_auth(&config).await?;

    if claims.game_id != game_id {
        return Err(AppError::Forbidden("Token is for a different game".to_string()).into());
    }
    if claims.role != TeamRole::Seeker && !claims.is_host {
        return Err(AppError::Forbidden("Only seekers can add exclusion zones".to_string()).into());
    }

    let radius = req.radius_m as i32;

    let row = sqlx::query(
        "INSERT INTO exclusion_zones (game_id, team_id, question_id, center_lat, center_lng, radius_m, exclude_outside, label)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, created_at",
    )
    .bind(game_id)
    .bind(claims.team_id)
    .bind(req.question_id)
    .bind(req.center_lat)
    .bind(req.center_lng)
    .bind(radius)
    .bind(req.exclude_outside)
    .bind(&req.label)
    .fetch_one(&pool)
    .await
    .map_err(AppError::Database)?;

    let zone = ExclusionZone {
        id: row.try_get("id").unwrap_or_default(),
        game_id,
        team_id: claims.team_id,
        center_lat: req.center_lat,
        center_lng: req.center_lng,
        radius_m: radius,
        exclude_outside: req.exclude_outside,
        label: req.label,
        created_at: row.try_get("created_at").unwrap_or_default(),
    };

    hub.broadcast(game_id, ServerMessage::ZoneAdded { zone: zone.clone() }).await;

    log::info!("Zone added: game={game_id} zone={}", zone.id);

    Ok(zone)
}

/// List all exclusion zones for a game (seeker or host only).
#[server(endpoint = "/zones/list")]
pub async fn list_exclusion_zones(game_id: Uuid) -> Result<Vec<ExclusionZone>, ServerFnError> {
    use crate::AppError;
    use axum::extract::Extension;
    use sqlx::{PgPool, Row};
    use std::sync::Arc;

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let config: Extension<Arc<crate::config::Config>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;
    let claims = crate::auth::require_auth(&config).await?;

    if claims.game_id != game_id {
        return Err(AppError::Forbidden("Token is for a different game".to_string()).into());
    }
    if claims.role != TeamRole::Seeker && !claims.is_host {
        return Err(AppError::Forbidden("Only seekers can view exclusion zones".to_string()).into());
    }

    let zones = sqlx::query(
        "SELECT id, game_id, team_id, center_lat, center_lng, radius_m, exclude_outside, label, created_at
         FROM exclusion_zones WHERE game_id = $1 ORDER BY created_at",
    )
    .bind(game_id)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?
    .iter()
    .map(|r| ExclusionZone {
        id: r.try_get("id").unwrap_or_default(),
        game_id: r.try_get("game_id").unwrap_or_default(),
        team_id: r.try_get("team_id").unwrap_or_default(),
        center_lat: r.try_get("center_lat").unwrap_or_default(),
        center_lng: r.try_get("center_lng").unwrap_or_default(),
        radius_m: r.try_get("radius_m").unwrap_or_default(),
        exclude_outside: r.try_get("exclude_outside").unwrap_or_default(),
        label: r.try_get("label").ok(),
        created_at: r.try_get("created_at").unwrap_or_default(),
    })
    .collect();

    Ok(zones)
}

/// Remove an exclusion zone (must be the team that added it, or host).
#[server(endpoint = "/zones/remove")]
pub async fn remove_exclusion_zone(
    game_id: Uuid,
    zone_id: Uuid,
) -> Result<(), ServerFnError> {
    use crate::AppError;
    use axum::extract::Extension;
    use sqlx::{PgPool, Row};
    use std::sync::Arc;

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let config: Extension<Arc<crate::config::Config>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let hub: Extension<Arc<crate::ws::GameHub>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;
    let hub = hub.0;
    let claims = crate::auth::require_auth(&config).await?;

    if claims.game_id != game_id {
        return Err(AppError::Forbidden("Token is for a different game".to_string()).into());
    }

    let zone_row = sqlx::query(
        "SELECT team_id FROM exclusion_zones WHERE id = $1 AND game_id = $2",
    )
    .bind(zone_id)
    .bind(game_id)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("Zone {zone_id} not found")))?;

    let zone_team: Uuid = zone_row.try_get("team_id").map_err(AppError::Database)?;
    if zone_team != claims.team_id && !claims.is_host {
        return Err(AppError::Forbidden("Cannot remove another team's zone".to_string()).into());
    }

    sqlx::query("DELETE FROM exclusion_zones WHERE id = $1")
        .bind(zone_id)
        .execute(&pool)
        .await
        .map_err(AppError::Database)?;

    hub.broadcast(game_id, ServerMessage::ZoneRemoved { zone_id }).await;

    log::info!("Zone removed: game={game_id} zone={zone_id}");

    Ok(())
}
