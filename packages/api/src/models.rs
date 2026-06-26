use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Enums ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(type_name = "team_role", rename_all = "snake_case"))]
pub enum TeamRole {
    Hider,
    Seeker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(type_name = "game_status", rename_all = "snake_case"))]
pub enum GameStatus {
    Lobby,
    Active,
    Finished,
}

impl std::fmt::Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::Lobby => write!(f, "Lobby"),
            GameStatus::Active => write!(f, "Active"),
            GameStatus::Finished => write!(f, "Finished"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(type_name = "map_size", rename_all = "snake_case"))]
pub enum MapSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(type_name = "card_type", rename_all = "snake_case"))]
pub enum CardType {
    Bonus,
    Curse,
}

// ── Auth ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub map_id: Uuid,
    pub host_display_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGameResponse {
    pub game_code: String,
    pub game_id: Uuid,
    pub team_id: Uuid,
    pub player_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JoinGameRequest {
    pub game_code: String,
    pub display_name: String,
    pub team_name: String,
    pub role: TeamRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JoinGameResponse {
    pub game_id: Uuid,
    pub team_id: Uuid,
    pub player_id: Uuid,
    pub role: TeamRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub game_id: Uuid,
    pub team_id: Uuid,
    pub player_id: Uuid,
    pub role: TeamRole,
    pub is_host: bool,
    pub game_status: GameStatus,
}

// ── Maps ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapBounds {
    pub sw_lat: f64,
    pub sw_lng: f64,
    pub ne_lat: f64,
    pub ne_lng: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapSummary {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapStop {
    pub id: Uuid,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub stop_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapQuestion {
    pub id: Uuid,
    pub text: String,
    pub radius_m: Option<i32>,
    pub requires_stop: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapDetail {
    pub id: Uuid,
    pub name: String,
    pub size: MapSize,
    pub bounds: MapBounds,
    pub stops: Vec<MapStop>,
    pub questions: Vec<MapQuestion>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateStopRequest {
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub stop_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateQuestionRequest {
    pub text: String,
    pub radius_m: Option<i32>,
    pub requires_stop: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateMapRequest {
    pub name: String,
    pub size: MapSize,
    pub bounds: MapBounds,
    pub stops: Vec<CreateStopRequest>,
    pub questions: Vec<CreateQuestionRequest>,
}

// ── Game state ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub display_name: String,
    pub is_host: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamInfo {
    pub id: Uuid,
    pub name: String,
    pub role: TeamRole,
    pub players: Vec<PlayerInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurnInfo {
    pub id: Uuid,
    pub hiding_team_id: Uuid,
    pub turn_number: i32,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub id: Uuid,
    pub code: String,
    pub status: GameStatus,
    pub map_id: Uuid,
    pub current_turn: Option<TurnInfo>,
}

// ── Exclusion zones ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExclusionZone {
    pub id: Uuid,
    pub game_id: Uuid,
    pub team_id: Uuid,
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_m: i32,
    pub exclude_outside: bool,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddZoneRequest {
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_m: u32,
    pub exclude_outside: bool,
    pub label: Option<String>,
    pub question_id: Option<Uuid>,
}

// ── Cards ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub name: String,
    pub card_type: CardType,
    pub effect: String,
    pub flavor_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardSummary {
    pub id: Uuid,
    pub name: String,
    pub card_type: CardType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawnCard {
    pub id: Uuid,
    pub card: Card,
    pub drawn_at: DateTime<Utc>,
    pub played_at: Option<DateTime<Utc>>,
}

// ── WebSocket messages ─────────────────────────────────────────────────────

/// Messages sent from the server to connected WebSocket clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    ZoneAdded { zone: ExclusionZone },
    ZoneRemoved { zone_id: Uuid },
    CardsDrawn { team_id: Uuid, cards: Vec<CardSummary> },
    CardPlayed { team_id: Uuid, drawn_card_id: Uuid },
    GameStatusChanged { status: GameStatus },
    PlayerJoined { player: PlayerInfo, team_id: Uuid },
    TurnChanged { turn: TurnInfo },
    Ping,
}

/// Messages sent from a client to the WebSocket server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Pong,
    RequestZoneSync,
}
