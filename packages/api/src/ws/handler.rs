use crate::models::{ClientMessage, ServerMessage, TeamRole};
use crate::ws::GameHub;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Extension, Path},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use uuid::Uuid;

const PING_INTERVAL: Duration = Duration::from_secs(30);

pub async fn ws_handler(
    Path(game_id): Path<Uuid>,
    ws: WebSocketUpgrade,
    Extension(hub): Extension<Arc<GameHub>>,
    Extension(config): Extension<Arc<crate::config::Config>>,
    cookies: CookieJar,
) -> impl IntoResponse {
    let token = cookies.get("auth").map(|c| c.value().to_owned());

    let claims = token.and_then(|t| {
        crate::jwt::verify(&t, config.jwt_secret.as_bytes()).ok()
    });

    let Some(claims) = claims else {
        log::warn!("WS connection rejected: missing or invalid auth cookie game={game_id}");
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };

    if claims.game_id != game_id {
        log::warn!("WS connection rejected: token game mismatch game={game_id}");
        return axum::http::StatusCode::FORBIDDEN.into_response();
    }

    let role = claims.role;
    let team_id = claims.team_id;
    let is_host = claims.is_host;

    log::info!("WS connected: game={game_id} team={team_id} role={role:?} host={is_host}");

    ws.on_upgrade(move |socket| handle_socket(socket, game_id, team_id, role, is_host, hub))
        .into_response()
}

async fn handle_socket(
    socket: WebSocket,
    game_id: Uuid,
    team_id: Uuid,
    role: TeamRole,
    is_host: bool,
    hub: Arc<GameHub>,
) {
    let mut rx = hub.subscribe(game_id).await;
    let (mut sender, mut receiver) = socket.split();
    let mut ping_ticker = interval(PING_INTERVAL);

    loop {
        tokio::select! {
            // Outbound: broadcast messages from hub, filtered by role
            msg = rx.recv() => {
                match msg {
                    Ok(server_msg) => {
                        if should_send(&server_msg, role, team_id, is_host) {
                            let text = match serde_json::to_string(&server_msg) {
                                Ok(t) => t,
                                Err(e) => {
                                    log::error!("WS serialize error: {e}");
                                    continue;
                                }
                            };
                            if sender.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        log::warn!("WS lagged {n} messages: game={game_id} team={team_id}");
                        // Reconnect will re-sync via HTTP
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }

            // Inbound: messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(text.as_str()) {
                            Ok(ClientMessage::Pong) => {}
                            Ok(ClientMessage::RequestZoneSync) => {
                                // Handled by client re-fetching zones via HTTP
                            }
                            Err(e) => {
                                log::warn!("WS bad client message: {e}");
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => {
                        log::warn!("WS receive error: {e}");
                        break;
                    }
                    _ => {}
                }
            }

            // Keepalive ping
            _ = ping_ticker.tick() => {
                let ping = serde_json::to_string(&ServerMessage::Ping).unwrap_or_default();
                if sender.send(Message::Text(ping.into())).await.is_err() {
                    break;
                }
            }
        }
    }

    log::info!("WS disconnected: game={game_id} team={team_id}");
    hub.cleanup(game_id).await;
}

fn should_send(msg: &ServerMessage, role: TeamRole, team_id: Uuid, is_host: bool) -> bool {
    if is_host {
        return true; // host sees everything
    }
    match msg {
        ServerMessage::ZoneAdded { .. } | ServerMessage::ZoneRemoved { .. } => {
            role == TeamRole::Seeker
        }
        ServerMessage::CardsDrawn { team_id: tid, .. }
        | ServerMessage::CardPlayed { team_id: tid, .. } => *tid == team_id,
        // Everyone sees these
        ServerMessage::GameStatusChanged { .. }
        | ServerMessage::PlayerJoined { .. }
        | ServerMessage::TurnChanged { .. }
        | ServerMessage::Ping => true,
    }
}
