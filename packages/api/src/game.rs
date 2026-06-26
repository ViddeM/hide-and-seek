use crate::models::*;
use dioxus::prelude::*;
use uuid::Uuid;

/// Get current game state (requires auth).
#[server(endpoint = "/games/state")]
pub async fn get_game_state(game_id: Uuid) -> Result<GameState, ServerFnError> {
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

    let row = sqlx::query(
        "SELECT id, code, status::text AS status, map_id FROM games WHERE id = $1",
    )
    .bind(game_id)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("Game {game_id} not found")))?;

    let status = match row.try_get::<String, _>("status").as_deref() {
        Ok("active") => GameStatus::Active,
        Ok("finished") => GameStatus::Finished,
        _ => GameStatus::Lobby,
    };

    let current_turn = sqlx::query(
        "SELECT id, hiding_team, turn_number, started_at FROM turns
         WHERE game_id = $1 AND ended_at IS NULL
         ORDER BY turn_number DESC LIMIT 1",
    )
    .bind(game_id)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?
    .map(|r| TurnInfo {
        id: r.try_get("id").unwrap_or_default(),
        hiding_team_id: r.try_get("hiding_team").unwrap_or_default(),
        turn_number: r.try_get("turn_number").unwrap_or_default(),
        started_at: r.try_get("started_at").unwrap_or_default(),
    });

    Ok(GameState {
        id: row.try_get("id").unwrap_or_default(),
        code: row.try_get::<String, _>("code").unwrap_or_default().trim().to_string(),
        status,
        map_id: row.try_get("map_id").unwrap_or_default(),
        current_turn,
    })
}

/// List all teams and their players for a game.
#[server(endpoint = "/games/teams")]
pub async fn list_teams(game_id: Uuid) -> Result<Vec<TeamInfo>, ServerFnError> {
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

    let teams =
        sqlx::query("SELECT id, name, role::text AS role FROM teams WHERE game_id = $1 ORDER BY name")
            .bind(game_id)
            .fetch_all(&pool)
            .await
            .map_err(AppError::Database)?;

    let mut result = Vec::with_capacity(teams.len());
    for team in &teams {
        let team_id: Uuid = team.try_get("id").unwrap_or_default();

        let players = sqlx::query(
            "SELECT id, display_name, is_host FROM players WHERE team_id = $1 ORDER BY joined_at",
        )
        .bind(team_id)
        .fetch_all(&pool)
        .await
        .map_err(AppError::Database)?
        .iter()
        .map(|p| PlayerInfo {
            id: p.try_get("id").unwrap_or_default(),
            display_name: p.try_get("display_name").unwrap_or_default(),
            is_host: p.try_get("is_host").unwrap_or_default(),
        })
        .collect();

        let role = match team.try_get::<String, _>("role").as_deref() {
            Ok("seeker") => TeamRole::Seeker,
            _ => TeamRole::Hider,
        };

        result.push(TeamInfo {
            id: team_id,
            name: team.try_get("name").unwrap_or_default(),
            role,
            players,
        });
    }

    Ok(result)
}

/// Start a new turn (host-only).
#[server(endpoint = "/games/turn/start")]
pub async fn start_turn(game_id: Uuid, hiding_team_id: Uuid) -> Result<TurnInfo, ServerFnError> {
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

    if !claims.is_host || claims.game_id != game_id {
        return Err(AppError::Forbidden("Only the game host can start turns".to_string()).into());
    }

    // End any current turn
    sqlx::query(
        "UPDATE turns SET ended_at = now() WHERE game_id = $1 AND ended_at IS NULL",
    )
    .bind(game_id)
    .execute(&pool)
    .await
    .map_err(AppError::Database)?;

    let turn_number: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(MAX(turn_number), 0) FROM turns WHERE game_id = $1",
    )
    .bind(game_id)
    .fetch_one(&pool)
    .await
    .map_err(AppError::Database)?;

    let row = sqlx::query(
        "INSERT INTO turns (game_id, hiding_team, turn_number) VALUES ($1, $2, $3) RETURNING id, started_at",
    )
    .bind(game_id)
    .bind(hiding_team_id)
    .bind((turn_number + 1) as i32)
    .fetch_one(&pool)
    .await
    .map_err(AppError::Database)?;

    let turn = TurnInfo {
        id: row.try_get("id").unwrap_or_default(),
        hiding_team_id,
        turn_number: (turn_number + 1) as i32,
        started_at: row.try_get("started_at").unwrap_or_default(),
    };

    log::info!("Turn started: game={game_id} turn={} hiding={hiding_team_id}", turn.turn_number);

    Ok(turn)
}
