use crate::models::*;
use dioxus::prelude::*;
use uuid::Uuid;

/// Draw N random cards from the deck (hider-only, 1–5 cards).
#[server(endpoint = "/cards/draw")]
pub async fn draw_cards(game_id: Uuid, count: u8) -> Result<Vec<DrawnCard>, ServerFnError> {
    use crate::AppError;
    use axum::extract::Extension;
    use rand::seq::IndexedRandom;
    use sqlx::{PgPool, Row};
    use std::sync::Arc;

    if count == 0 || count > 5 {
        return Err(ServerFnError::new("count must be between 1 and 5"));
    }

    let pool: Extension<PgPool> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let config: Extension<Arc<crate::config::Config>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let hub: Extension<Arc<crate::ws::GameHub>> =
        dioxus::fullstack::FullstackContext::extract().await.map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = pool.0;
    let hub = hub.0;
    let claims = crate::auth::require_auth(&config).await?;

    if claims.game_id != game_id || claims.role != TeamRole::Hider {
        return Err(AppError::Forbidden("Only hiders can draw cards".to_string()).into());
    }

    // Fetch all card IDs and pick N at random (with replacement)
    let card_ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM cards")
        .fetch_all(&pool)
        .await
        .map_err(AppError::Database)?;

    if card_ids.is_empty() {
        return Err(AppError::Internal("Card deck is empty".to_string()).into());
    }

    let mut rng = rand::rng();
    let chosen: Vec<Uuid> = (0..count as usize)
        .map(|_| *card_ids.choose(&mut rng).expect("non-empty"))
        .collect();

    let mut drawn = Vec::with_capacity(count as usize);
    let mut summaries = Vec::with_capacity(count as usize);

    for card_id in &chosen {
        let drawn_row = sqlx::query(
            "INSERT INTO drawn_cards (game_id, team_id, card_id) VALUES ($1, $2, $3) RETURNING id, drawn_at",
        )
        .bind(game_id)
        .bind(claims.team_id)
        .bind(card_id)
        .fetch_one(&pool)
        .await
        .map_err(AppError::Database)?;

        let card_row = sqlx::query(
            "SELECT id, name, card_type::text AS card_type, effect, flavor_text FROM cards WHERE id = $1",
        )
        .bind(card_id)
        .fetch_one(&pool)
        .await
        .map_err(AppError::Database)?;

        let card_type = parse_card_type(&card_row.try_get::<String, _>("card_type").unwrap_or_default());
        let card = Card {
            id: card_row.try_get("id").unwrap_or_default(),
            name: card_row.try_get("name").unwrap_or_default(),
            card_type,
            effect: card_row.try_get("effect").unwrap_or_default(),
            flavor_text: card_row.try_get("flavor_text").ok(),
        };

        summaries.push(CardSummary {
            id: card.id,
            name: card.name.clone(),
            card_type,
        });

        drawn.push(DrawnCard {
            id: drawn_row.try_get("id").unwrap_or_default(),
            card,
            drawn_at: drawn_row.try_get("drawn_at").unwrap_or_default(),
            played_at: None,
        });
    }

    hub.broadcast(
        game_id,
        ServerMessage::CardsDrawn { team_id: claims.team_id, cards: summaries },
    )
    .await;

    log::info!("Cards drawn: game={game_id} team={} count={count}", claims.team_id);

    Ok(drawn)
}

/// Mark a drawn card as played (hider-only).
#[server(endpoint = "/cards/play")]
pub async fn play_card(game_id: Uuid, drawn_card_id: Uuid) -> Result<(), ServerFnError> {
    use crate::AppError;
    use axum::extract::Extension;
    use sqlx::PgPool;
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

    if claims.game_id != game_id || claims.role != TeamRole::Hider {
        return Err(AppError::Forbidden("Only hiders can play cards".to_string()).into());
    }

    let rows = sqlx::query(
        "UPDATE drawn_cards SET played_at = now()
         WHERE id = $1 AND team_id = $2 AND game_id = $3 AND played_at IS NULL",
    )
    .bind(drawn_card_id)
    .bind(claims.team_id)
    .bind(game_id)
    .execute(&pool)
    .await
    .map_err(AppError::Database)?;

    if rows.rows_affected() == 0 {
        return Err(AppError::NotFound("Card not found or already played".to_string()).into());
    }

    hub.broadcast(
        game_id,
        ServerMessage::CardPlayed { team_id: claims.team_id, drawn_card_id },
    )
    .await;

    log::info!("Card played: game={game_id} card={drawn_card_id}");

    Ok(())
}

/// Get the current hand (unplayed drawn cards) for the calling team.
#[server(endpoint = "/cards/hand")]
pub async fn get_hand(game_id: Uuid) -> Result<Vec<DrawnCard>, ServerFnError> {
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

    if claims.game_id != game_id || claims.role != TeamRole::Hider {
        return Err(AppError::Forbidden("Only hiders can view their hand".to_string()).into());
    }

    let rows = sqlx::query(
        "SELECT dc.id, dc.drawn_at, dc.played_at,
                c.id AS card_id, c.name AS card_name, c.card_type::text AS card_type,
                c.effect, c.flavor_text
         FROM drawn_cards dc
         JOIN cards c ON c.id = dc.card_id
         WHERE dc.game_id = $1 AND dc.team_id = $2 AND dc.played_at IS NULL
         ORDER BY dc.drawn_at",
    )
    .bind(game_id)
    .bind(claims.team_id)
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    let hand = rows
        .iter()
        .map(|r| {
            let card_type = parse_card_type(&r.try_get::<String, _>("card_type").unwrap_or_default());
            DrawnCard {
                id: r.try_get("id").unwrap_or_default(),
                card: Card {
                    id: r.try_get("card_id").unwrap_or_default(),
                    name: r.try_get("card_name").unwrap_or_default(),
                    card_type,
                    effect: r.try_get("effect").unwrap_or_default(),
                    flavor_text: r.try_get("flavor_text").ok(),
                },
                drawn_at: r.try_get("drawn_at").unwrap_or_default(),
                played_at: r.try_get("played_at").ok(),
            }
        })
        .collect();

    Ok(hand)
}

#[cfg(feature = "server")]
fn parse_card_type(s: &str) -> CardType {
    match s {
        "curse" => CardType::Curse,
        _ => CardType::Bonus,
    }
}
