use crate::models::TeamRole;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const JWT_EXPIRY_SECS: i64 = 60 * 60 * 24 * 7; // 7 days

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,      // player_id
    pub game_id: Uuid,
    pub team_id: Uuid,
    pub role: TeamRole,
    pub is_host: bool,
    pub exp: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("invalid token: {0}")]
    Invalid(#[from] jsonwebtoken::errors::Error),
    #[error("token missing from cookie jar")]
    Missing,
}

impl Claims {
    pub fn new(
        player_id: Uuid,
        game_id: Uuid,
        team_id: Uuid,
        role: TeamRole,
        is_host: bool,
    ) -> Self {
        let exp = chrono::Utc::now().timestamp() + JWT_EXPIRY_SECS;
        Self {
            sub: player_id,
            game_id,
            team_id,
            role,
            is_host,
            exp,
        }
    }
}

pub fn sign(claims: &Claims, secret: &[u8]) -> Result<String, JwtError> {
    let token = encode(&Header::default(), claims, &EncodingKey::from_secret(secret))?;
    Ok(token)
}

pub fn verify(token: &str, secret: &[u8]) -> Result<Claims, JwtError> {
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &Validation::default())?;
    Ok(data.claims)
}
