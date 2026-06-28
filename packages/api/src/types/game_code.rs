use std::{
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

/// A game code, which is always an 8 character hexadecimal string.
///
/// You can create a [GameCode] by calling `.parse` on a string.
///
/// # Invariant
///
/// The inner `[u8; 8]` must always be a valid 8-character hexadecimal string. This is enforced
/// through the `From<u32>`, and the [FromStr] impls, which are the only valid ways to make GameCodes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct GameCode([u8; 8]);

#[cfg(feature = "server")]
impl GameCode {
    /// Generate a new random game code.
    pub fn random() -> Self {
        let id: u32 = rand::random();
        id.into()
    }
}

impl Deref for GameCode {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        str::from_utf8(&self.0).expect("Game Code must always be a hexadecimal str")
    }
}

impl FromStr for GameCode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: &[u8; 8] = s
            .as_bytes()
            .try_into()
            .map_err(|_| "Game Code must be exactly 8 bytes long")?;

        if s.chars().any(|c| !c.is_ascii_hexdigit()) {
            return Err("Game Code must be a hexadecimal string");
        }

        let _parsed = u32::from_str_radix(s, 16)
            .map_err(|_| "failed to parse game code, expected hex string of length 8")?;

        Ok(GameCode(*id))
    }
}

impl From<u32> for GameCode {
    fn from(id: u32) -> Self {
        let mut buf = [0u8; 8];
        let game_code = format!("{id:08x}");
        buf.copy_from_slice(game_code.as_bytes());
        GameCode(buf)
    }
}

impl Serialize for GameCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.deref().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GameCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl Debug for GameCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl Display for GameCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.deref(), f)
    }
}
