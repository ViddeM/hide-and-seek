use serde::{Deserialize, Serialize};

pub mod area;
pub mod game_code;
pub mod map_size;

pub type Meters = f64;
pub type Coordinate = f64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub lat: Coordinate,
    pub lng: Coordinate,
}
