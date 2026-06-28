pub mod game_code;
pub mod map_size;

pub type Coordinate = f64;

pub struct Point {
    pub lat: Coordinate,
    pub lng: Coordinate,
}
