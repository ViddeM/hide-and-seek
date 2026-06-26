use crate::models::*;
use dioxus::prelude::*;
use uuid::Uuid;

/// Create a new game. The caller becomes the host.
/// Sets an HTTP-only `auth` cookie with a host JWT.
#[server(endpoint = "/games/create")]
pub async fn create_game(
    map_id: Uuid,
    host_display_name: String,
) -> Result<CreateGameResponse, ServerFnError> {
    use crate::{jwt, AppError};
    use axum::extract::Extension;
    use rand::distr::{Alphanumeric, SampleString};
    use sqlx::PgPool;
    use std::sync::Arc;

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let config: Extension<Arc<crate::config::Config>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;
    let config = config.0;

    // Generate unique 6-char uppercase alphanumeric game code
    let code = {
        let mut code;
        loop {
            code = Alphanumeric
                .sample_string(&mut rand::rng(), 6)
                .to_uppercase();
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM games WHERE code = $1")
                .bind(&code)
                .fetch_one(&pool)
                .await
                .map_err(AppError::Database)?;
            if count == 0 {
                break;
            }
        }
        code
    };

    let game_id: Uuid =
        sqlx::query_scalar("INSERT INTO games (code, map_id) VALUES ($1, $2) RETURNING id")
            .bind(&code)
            .bind(map_id)
            .fetch_one(&pool)
            .await
            .map_err(AppError::Database)?;

    // Host gets a dedicated team
    let team_id: Uuid =
        sqlx::query_scalar("INSERT INTO teams (game_id, name, role) VALUES ($1, 'Host', 'hider') RETURNING id")
            .bind(game_id)
            .fetch_one(&pool)
            .await
            .map_err(AppError::Database)?;

    let player_id: Uuid = sqlx::query_scalar(
        "INSERT INTO players (team_id, display_name, is_host) VALUES ($1, $2, true) RETURNING id",
    )
    .bind(team_id)
    .bind(&host_display_name)
    .fetch_one(&pool)
    .await
    .map_err(AppError::Database)?;

    let claims = jwt::Claims::new(player_id, game_id, team_id, TeamRole::Hider, true);
    let token = jwt::sign(&claims, config.jwt_secret.as_bytes())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    set_auth_cookie(&token);

    log::info!("Game created: code={code} game_id={game_id} host={host_display_name}");

    Ok(CreateGameResponse { game_code: code, game_id, team_id, player_id })
}

/// Join an existing game as a player.
/// Sets an HTTP-only `auth` cookie.
#[server(endpoint = "/games/join")]
pub async fn join_game(
    game_code: String,
    display_name: String,
    team_name: String,
    role: TeamRole,
) -> Result<JoinGameResponse, ServerFnError> {
    use crate::{jwt, AppError};
    use axum::extract::Extension;
    use sqlx::{PgPool, Row};
    use std::sync::Arc;

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let config: Extension<Arc<crate::config::Config>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;
    let config = config.0;

    let code = game_code.to_uppercase();

    let game_row = sqlx::query("SELECT id, status::text AS status FROM games WHERE code = $1")
        .bind(&code)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Game with code {code} not found")))?;

    let status: String = game_row.try_get("status").map_err(AppError::Database)?;
    if status != "lobby" {
        return Err(AppError::BadRequest("Game has already started".to_string()).into());
    }
    let game_id: Uuid = game_row.try_get("id").map_err(AppError::Database)?;

    let role_str = match role {
        TeamRole::Hider => "hider",
        TeamRole::Seeker => "seeker",
    };

    // Get existing team or create new one (validate role consistency)
    let existing = sqlx::query("SELECT id, role::text AS role FROM teams WHERE game_id = $1 AND name = $2")
        .bind(game_id)
        .bind(&team_name)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::Database)?;

    let team_id: Uuid = if let Some(row) = existing {
        let existing_role: String = row.try_get("role").map_err(AppError::Database)?;
        if existing_role != role_str {
            return Err(AppError::BadRequest(format!(
                "Team '{team_name}' is a {existing_role} team, cannot join as {role_str}"
            ))
            .into());
        }
        row.try_get("id").map_err(AppError::Database)?
    } else {
        sqlx::query_scalar(
            "INSERT INTO teams (game_id, name, role) VALUES ($1, $2, $3::team_role) RETURNING id",
        )
        .bind(game_id)
        .bind(&team_name)
        .bind(role_str)
        .fetch_one(&pool)
        .await
        .map_err(AppError::Database)?
    };

    let player_id: Uuid =
        sqlx::query_scalar("INSERT INTO players (team_id, display_name) VALUES ($1, $2) RETURNING id")
            .bind(team_id)
            .bind(&display_name)
            .fetch_one(&pool)
            .await
            .map_err(AppError::Database)?;

    let claims = jwt::Claims::new(player_id, game_id, team_id, role, false);
    let token = jwt::sign(&claims, config.jwt_secret.as_bytes())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    set_auth_cookie(&token);

    log::info!(
        "Player joined: name={display_name} game={code} team={team_name} role={role_str}"
    );

    Ok(JoinGameResponse { game_id, team_id, player_id, role })
}

/// Start the game (host-only). Transitions lobby → active.
#[server(endpoint = "/games/start")]
pub async fn start_game(game_id: Uuid) -> Result<(), ServerFnError> {
    use crate::AppError;
    use axum::extract::Extension;
    use sqlx::PgPool;
    use std::sync::Arc;

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let config: Extension<Arc<crate::config::Config>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;
    let claims = require_auth(&config).await?;

    if !claims.is_host || claims.game_id != game_id {
        return Err(AppError::Forbidden("Only the host can start the game".to_string()).into());
    }

    sqlx::query(
        "UPDATE games SET status = 'active', started_at = now() WHERE id = $1 AND status = 'lobby'",
    )
    .bind(game_id)
    .execute(&pool)
    .await
    .map_err(AppError::Database)?;

    log::info!("Game started: game_id={game_id}");

    Ok(())
}

/// Return session info derived from the JWT cookie.
#[server(endpoint = "/session")]
pub async fn get_session() -> Result<Option<SessionInfo>, ServerFnError> {
    use crate::AppError;
    use axum::extract::Extension;
    use sqlx::{PgPool, Row};
    use std::sync::Arc;

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let config: Extension<Arc<crate::config::Config>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;

    let claims = match try_auth(&config).await? {
        Some(c) => c,
        None => return Ok(None),
    };

    let status_opt = sqlx::query("SELECT status::text AS status FROM games WHERE id = $1")
        .bind(claims.game_id)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::Database)?;

    let game_status = match status_opt {
        Some(row) => {
            let s: String = row.try_get("status").unwrap_or_default();
            match s.as_str() {
                "active" => GameStatus::Active,
                "finished" => GameStatus::Finished,
                _ => GameStatus::Lobby,
            }
        }
        None => return Ok(None),
    };

    Ok(Some(SessionInfo {
        game_id: claims.game_id,
        team_id: claims.team_id,
        player_id: claims.sub,
        role: claims.role,
        is_host: claims.is_host,
        game_status,
    }))
}

// ── Helpers (server-only) ──────────────────────────────────────────────────

#[cfg(feature = "server")]
pub(crate) fn set_auth_cookie(token: &str) {
    let cookie_value = format!("auth={token}; HttpOnly; SameSite=Lax; Path=/");
    if let Some(ctx) = dioxus::fullstack::FullstackContext::current() {
        if let Ok(header_val) = axum::http::HeaderValue::try_from(cookie_value) {
            ctx.add_response_header(axum::http::header::SET_COOKIE, header_val);
        }
    }
}

#[cfg(feature = "server")]
pub(crate) async fn require_auth(
    config: &axum::extract::Extension<std::sync::Arc<crate::config::Config>>,
) -> Result<crate::jwt::Claims, ServerFnError> {
    try_auth(config)
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))
}

#[cfg(feature = "server")]
pub(crate) async fn try_auth(
    config: &axum::extract::Extension<std::sync::Arc<crate::config::Config>>,
) -> Result<Option<crate::jwt::Claims>, ServerFnError> {
    use crate::{jwt, AppError};
    use axum_extra::extract::CookieJar;

    let jar: CookieJar = dioxus::fullstack::FullstackContext::extract().await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let Some(cookie) = jar.get("auth") else {
        return Ok(None);
    };

    let claims = jwt::verify(cookie.value(), config.jwt_secret.as_bytes())
        .map_err(|e| AppError::Auth(e.to_string()))?;

    Ok(Some(claims))
}
