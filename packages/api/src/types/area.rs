use serde::{Deserialize, Serialize};

use crate::types::{Meters, Point};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: Point,
    pub radius: Meters,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    pub vertices: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Area {
    Circle(Circle),
    Line(Line),
    Polygon(Polygon),
}

impl Area {
    pub fn display(&self) -> String {
        match self {
            Area::Circle(circle) => format!("Circle {}m radius", circle.radius),
            Area::Line(line) => format!("Line of length {}m", line.start.distance_to(&line.end)),
            Area::Polygon(polygon) => format!("Polygon with {} vertices", polygon.vertices.len()),
        }
    }
}
