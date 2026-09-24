use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::{
    Point,
    transit::{TransitData, TransitRoute, TransitRouteType, TransitStop},
};

pub async fn insert_transit_data(
    pool: &PgPool,
    map_id: Uuid,
    routes: &[TransitRoute],
) -> Result<(), sqlx::Error> {
    for route in routes {
        // Stored as array-of-segments: [[[lat,lng],...], ...]
        let shape: Vec<Vec<[f64; 2]>> = route.waypoints.iter()
            .map(|seg| seg.iter().map(|p| [p.lat, p.lng]).collect())
            .collect();
        let shape_json = serde_json::to_value(&shape).unwrap_or(JsonValue::Array(vec![]));

        sqlx::query(
            "INSERT INTO transit_routes (id, map_id, name, long_name, route_type, color, shape)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(route.id)
        .bind(map_id)
        .bind(&route.name)
        .bind(&route.long_name)
        .bind(&route.route_type)
        .bind(&route.color)
        .bind(&shape_json)
        .execute(pool)
        .await?;

        for (seq, stop) in route.stops.iter().enumerate() {
            sqlx::query(
                "INSERT INTO transit_stops (id, map_id, name, lat, lng) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(stop.id)
            .bind(map_id)
            .bind(&stop.name)
            .bind(stop.lat)
            .bind(stop.lng)
            .execute(pool)
            .await?;

            sqlx::query(
                "INSERT INTO transit_route_stops (route_id, stop_id, seq) VALUES ($1, $2, $3)",
            )
            .bind(route.id)
            .bind(stop.id)
            .bind(seq as i32)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

#[derive(sqlx::FromRow)]
struct RouteRow {
    id: Uuid,
    name: String,
    long_name: Option<String>,
    route_type: TransitRouteType,
    color: Option<String>,
    shape: JsonValue,
}

#[derive(sqlx::FromRow)]
struct StopRow {
    id: Uuid,
    name: String,
    lat: f64,
    lng: f64,
}

pub async fn get_transit_data(pool: &PgPool, map_id: Uuid) -> Result<TransitData, sqlx::Error> {
    let route_rows: Vec<RouteRow> = sqlx::query_as(
        "SELECT id, name, long_name, route_type, color, shape
         FROM transit_routes WHERE map_id = $1 ORDER BY name",
    )
    .bind(map_id)
    .fetch_all(pool)
    .await?;

    let mut routes = Vec::new();
    for row in route_rows {
        // shape is stored as array-of-segments: [[[lat,lng],...], ...]
        let waypoints: Vec<Vec<Point>> = row
            .shape
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|seg| {
                seg.as_array().unwrap_or(&vec![]).iter()
                    .filter_map(|v| {
                        let arr = v.as_array()?;
                        Some(Point { lat: arr.first()?.as_f64()?, lng: arr.get(1)?.as_f64()? })
                    })
                    .collect()
            })
            .collect();

        let stop_rows: Vec<StopRow> = sqlx::query_as(
            "SELECT ts.id, ts.name, ts.lat, ts.lng
             FROM transit_stops ts
             JOIN transit_route_stops trs ON trs.stop_id = ts.id
             WHERE trs.route_id = $1
             ORDER BY trs.seq",
        )
        .bind(row.id)
        .fetch_all(pool)
        .await?;

        let stops = stop_rows
            .into_iter()
            .map(|s| TransitStop { id: s.id, name: s.name, lat: s.lat, lng: s.lng })
            .collect();

        routes.push(TransitRoute {
            id: row.id,
            name: row.name,
            long_name: row.long_name,
            route_type: row.route_type,
            color: row.color,
            waypoints,
            stops,
        });
    }

    Ok(TransitData { routes })
}
