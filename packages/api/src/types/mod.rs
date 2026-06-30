use serde::{Deserialize, Serialize};

pub mod area;
pub mod game_code;
pub mod game_status;
pub mod map_size;

pub type Meters = f64;
pub type Coordinate = f64;

const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub lat: Coordinate,
    pub lng: Coordinate,
}

impl Point {
    pub fn distance_to(&self, other: &Point) -> Meters {
        let lat1 = self.lat.to_radians();
        let lat2 = other.lat.to_radians();
        let delta_lat = (other.lat - self.lat).to_radians();
        let delta_lng = (other.lng - self.lng).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_METERS * c
    }
}
