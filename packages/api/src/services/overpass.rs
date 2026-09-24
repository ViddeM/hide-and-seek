use std::collections::HashMap;

use serde::Deserialize;

use crate::types::{
    Point,
    area::Polygon,
    map_size::MapSize,
    transit::{TransitRoute, TransitRouteType, TransitStop},
};

const OVERPASS_MIRRORS: &[&str] = &[
    "https://overpass-api.de/api/interpreter",
    "https://overpass.kumi.systems/api/interpreter",
    "https://overpass.openstreetmap.ru/api/interpreter",
];

fn route_types_for_size(size: MapSize) -> &'static str {
    match size {
        MapSize::Small => "bus|tram|subway|trolleybus|monorail|light_rail|ferry|funicular",
        MapSize::Medium => "bus|tram|subway|trolleybus|light_rail|monorail|train|ferry",
        MapSize::Large => "train|ferry",
    }
}

fn parse_route_type(s: &str) -> Option<TransitRouteType> {
    match s {
        "bus" => Some(TransitRouteType::Bus),
        "tram" => Some(TransitRouteType::Tram),
        "subway" | "metro" => Some(TransitRouteType::Subway),
        "train" => Some(TransitRouteType::Train),
        "ferry" => Some(TransitRouteType::Ferry),
        "monorail" => Some(TransitRouteType::Monorail),
        "light_rail" => Some(TransitRouteType::LightRail),
        "trolleybus" => Some(TransitRouteType::Trolleybus),
        "funicular" => Some(TransitRouteType::Funicular),
        _ => None,
    }
}

fn point_in_polygon(point: &Point, polygon: &[Point]) -> bool {
    let n = polygon.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let xi = polygon[i].lng;
        let yi = polygon[i].lat;
        let xj = polygon[j].lng;
        let yj = polygon[j].lat;
        if ((yi > point.lat) != (yj > point.lat))
            && (point.lng < (xj - xi) * (point.lat - yi) / (yj - yi) + xi)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn round5(v: f64) -> f64 {
    (v * 100_000.0).round() / 100_000.0
}

fn thin_segment(seg: Vec<Point>, max_pts: usize) -> Vec<Point> {
    if seg.len() <= max_pts {
        return seg;
    }
    let step = (seg.len() as f64 / max_pts as f64).ceil() as usize;
    let mut out: Vec<Point> = seg.iter().step_by(step).cloned().collect();
    if let Some(last) = seg.last() {
        if out.last() != Some(last) {
            out.push(last.clone());
        }
    }
    out
}

fn build_poly_string(vertices: &[Point]) -> String {
    vertices
        .iter()
        .map(|p| format!("{} {}", p.lat, p.lng))
        .collect::<Vec<_>>()
        .join(" ")
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            b => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ── Overpass JSON response types ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct OverpassResponse {
    elements: Vec<Element>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Element {
    Node(NodeElement),
    Way(WayElement),
    Relation(RelationElement),
}

#[derive(Debug, Deserialize)]
struct NodeElement {
    id: i64,
    lat: f64,
    lon: f64,
    #[serde(default)]
    tags: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct WayElement {
    #[allow(dead_code)]
    id: i64,
    nodes: Vec<i64>,
}

#[derive(Debug, Deserialize)]
struct RelationElement {
    id: i64,
    tags: HashMap<String, String>,
    members: Vec<Member>,
}

#[derive(Debug, Deserialize)]
struct Member {
    #[serde(rename = "type")]
    member_type: String,
    #[serde(rename = "ref")]
    member_ref: i64,
    #[serde(default)]
    role: String,
}

pub async fn fetch_transit(
    size: MapSize,
    bounds: &Polygon,
) -> Result<Vec<TransitRoute>, anyhow::Error> {
    let poly_str = build_poly_string(&bounds.vertices);
    let types = route_types_for_size(size);

    let query = format!(
        r#"[out:json][timeout:20];
(
  relation["type"="route"]["route"~"{types}"](poly:"{poly_str}");
);
out body;
>;
out body qt;"#
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .user_agent("hide-and-seek-game/1.0 (contact: vidar.magnusson@accenture.com)")
        .build()?;

    let body = format!("data={}", urlencode(&query));
    let mut last_err = anyhow::anyhow!("no Overpass mirrors configured");

    let response_text = 'mirrors: {
        for url in OVERPASS_MIRRORS {
            match client
                .post(*url)
                .body(body.clone())
                .header("Content-Type", "application/x-www-form-urlencoded")
                .send()
                .await
            {
                Ok(resp) => match resp.error_for_status() {
                    Ok(ok) => break 'mirrors ok.text().await?,
                    Err(e) => {
                        tracing::warn!("Overpass mirror {url} returned error: {e}");
                        last_err = e.into();
                    }
                },
                Err(e) => {
                    tracing::warn!("Overpass mirror {url} unreachable: {e}");
                    last_err = e.into();
                }
            }
        }
        return Err(last_err);
    };

    let overpass: OverpassResponse = serde_json::from_str(&response_text)?;

    let mut nodes: HashMap<i64, NodeElement> = HashMap::new();
    let mut ways: HashMap<i64, WayElement> = HashMap::new();
    let mut relations: Vec<RelationElement> = Vec::new();

    for element in overpass.elements {
        match element {
            Element::Node(n) => {
                nodes.insert(n.id, n);
            }
            Element::Way(w) => {
                ways.insert(w.id, w);
            }
            Element::Relation(r) => relations.push(r),
        }
    }

    let vertices = &bounds.vertices;
    let mut routes: Vec<TransitRoute> = Vec::new();

    let stop_roles = [
        "stop",
        "stop_entry_only",
        "stop_exit_only",
        "platform",
        "platform_entry_only",
        "platform_exit_only",
    ];

    for relation in relations {
        let route_tag = relation.tags.get("route").map(|s| s.as_str()).unwrap_or("");
        let Some(route_type) = parse_route_type(route_tag) else {
            continue;
        };

        let name = relation
            .tags
            .get("ref")
            .or_else(|| relation.tags.get("name"))
            .cloned()
            .unwrap_or_else(|| format!("Route {}", relation.id));

        let long_name = relation
            .tags
            .get("name")
            .filter(|n| *n != &name)
            .cloned();

        let color = relation
            .tags
            .get("colour")
            .or_else(|| relation.tags.get("color"))
            .cloned();

        let mut stop_node_ids: Vec<i64> = Vec::new();
        let mut way_member_ids: Vec<i64> = Vec::new();

        for member in &relation.members {
            if member.member_type == "node" && stop_roles.contains(&member.role.as_str()) {
                stop_node_ids.push(member.member_ref);
            } else if member.member_type == "way" {
                way_member_ids.push(member.member_ref);
            }
        }

        let in_boundary_stops: Vec<TransitStop> = stop_node_ids
            .iter()
            .filter_map(|id| {
                let node = nodes.get(id)?;
                let pt = Point { lat: node.lat, lng: node.lon };
                if !point_in_polygon(&pt, vertices) {
                    return None;
                }
                let stop_name = node
                    .tags
                    .get("name")
                    .cloned()
                    .unwrap_or_else(|| format!("Stop {}", node.id));
                Some(TransitStop {
                    id: uuid::Uuid::new_v4(),
                    name: stop_name,
                    lat: round5(node.lat),
                    lng: round5(node.lon),
                })
            })
            .collect();

        if in_boundary_stops.len() < 2 {
            continue;
        }

        // Reconstruct geometry: stitch way segments, reversing as needed, splitting on gaps.
        // SNAP_THRESHOLD is in degrees-squared; ~0.001° ≈ 100 m, generous for ferry terminals.
        const SNAP_THRESHOLD: f64 = 1e-6;
        fn dist_sq(a: &Point, b: &Point) -> f64 {
            (a.lat - b.lat).powi(2) + (a.lng - b.lng).powi(2)
        }

        let mut segments: Vec<Vec<Point>> = Vec::new();
        let mut current: Vec<Point> = Vec::new();

        for way_id in &way_member_ids {
            if let Some(way) = ways.get(way_id) {
                let mut seg: Vec<Point> = way
                    .nodes
                    .iter()
                    .filter_map(|nid| nodes.get(nid))
                    .map(|n| Point { lat: round5(n.lat), lng: round5(n.lon) })
                    .collect();

                if seg.is_empty() {
                    continue;
                }

                if current.is_empty() {
                    current = seg;
                    continue;
                }

                let last = current.last().unwrap().clone();
                let d_fwd = dist_sq(&last, seg.first().unwrap());
                let d_rev = dist_sq(&last, seg.last().unwrap());

                if d_fwd <= d_rev {
                    if d_fwd < SNAP_THRESHOLD {
                        current.extend_from_slice(&seg[1..]);
                    } else {
                        // True gap — start a new segment
                        segments.push(std::mem::take(&mut current));
                        current = seg;
                    }
                } else {
                    // Way is stored in reverse — flip it
                    seg.reverse();
                    let d_after_flip = dist_sq(&last, seg.first().unwrap());
                    if d_after_flip < SNAP_THRESHOLD {
                        current.extend_from_slice(&seg[1..]);
                    } else {
                        segments.push(std::mem::take(&mut current));
                        current = seg;
                    }
                }
            }
        }
        if !current.is_empty() {
            segments.push(current);
        }
        let waypoints: Vec<Vec<Point>> = segments
            .into_iter()
            .map(|seg| thin_segment(seg, 200))
            .collect();

        routes.push(TransitRoute {
            id: uuid::Uuid::new_v4(),
            name,
            long_name,
            route_type,
            color,
            waypoints,
            stops: in_boundary_stops,
        });
    }

    routes.truncate(150);
    Ok(routes)
}
