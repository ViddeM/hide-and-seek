use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Point;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(type_name = "transit_route_type", rename_all = "snake_case"))]
pub enum TransitRouteType {
    Bus,
    Tram,
    Subway,
    Train,
    Ferry,
    Monorail,
    LightRail,
    Trolleybus,
    Funicular,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitStop {
    pub id: Uuid,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitRoute {
    pub id: Uuid,
    pub name: String,
    pub long_name: Option<String>,
    pub route_type: TransitRouteType,
    pub color: Option<String>,
    pub waypoints: Vec<Point>,
    pub stops: Vec<TransitStop>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitData {
    pub routes: Vec<TransitRoute>,
}
