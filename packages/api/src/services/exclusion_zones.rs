use sqlx::PgPool;
use uuid::Uuid;

use crate::db::queries;
use crate::error::AppError;
use crate::types::Point;
use crate::types::area::{Area, Circle, Line, Polygon};

#[derive(Debug, Clone, PartialEq)]
pub struct ExclusionZoneDetail {
    pub id: Uuid,
    pub exclude_outside: bool,
    pub label: Option<String>,
    pub area: Area,
}

pub async fn list_exclusion_zones(
    pool: &PgPool,
    game_id: Uuid,
) -> Result<Vec<ExclusionZoneDetail>, AppError> {
    let rows = queries::exclusion_zones::get_zones_for_game(pool, game_id)
        .await
        .map_err(AppError::from)?;

    let mut result = Vec::with_capacity(rows.len());

    for row in rows {
        let area = build_area(&row, pool).await?;
        result.push(ExclusionZoneDetail {
            id: row.id,
            exclude_outside: row.exclude_outside,
            label: row.label,
            area,
        });
    }

    Ok(result)
}

pub async fn create_exclusion_zone(
    pool: &PgPool,
    game_id: Uuid,
    exclude_outside: bool,
    label: Option<String>,
    area: Area,
) -> Result<ExclusionZoneDetail, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;

    let area_id = match &area {
        Area::Circle(c) => {
            let circle_id = queries::exclusion_zones::insert_circle(
                &mut *tx,
                c.center.lat,
                c.center.lng,
                c.radius as i32,
            )
            .await
            .map_err(AppError::from)?;
            queries::exclusion_zones::insert_area_for_circle(&mut *tx, circle_id)
                .await
                .map_err(AppError::from)?
        }
        Area::Line(l) => {
            let line_id = queries::exclusion_zones::insert_line(
                &mut *tx,
                l.start.lat,
                l.start.lng,
                l.end.lat,
                l.end.lng,
            )
            .await
            .map_err(AppError::from)?;
            queries::exclusion_zones::insert_area_for_line(&mut *tx, line_id)
                .await
                .map_err(AppError::from)?
        }
        Area::Polygon(p) => {
            let polygon_id = queries::maps::insert_polygon(&mut *tx)
                .await
                .map_err(AppError::from)?;
            queries::maps::insert_polygon_points(&mut *tx, polygon_id, &p.vertices)
                .await
                .map_err(AppError::from)?;
            queries::exclusion_zones::insert_area_for_polygon(&mut *tx, polygon_id)
                .await
                .map_err(AppError::from)?
        }
    };

    let zone_id = queries::exclusion_zones::insert_exclusion_zone(
        &mut *tx,
        game_id,
        area_id,
        exclude_outside,
        &label,
    )
    .await
    .map_err(AppError::from)?;

    tx.commit().await.map_err(AppError::from)?;

    Ok(ExclusionZoneDetail {
        id: zone_id,
        exclude_outside,
        label,
        area,
    })
}

pub async fn remove_exclusion_zone(
    pool: &PgPool,
    game_id: Uuid,
    zone_id: Uuid,
) -> Result<(), AppError> {
    let row = queries::exclusion_zones::get_zone_for_game(pool, game_id, zone_id)
        .await
        .map_err(AppError::from)?;

    let mut tx = pool.begin().await.map_err(AppError::from)?;

    queries::exclusion_zones::delete_exclusion_zone(&mut *tx, zone_id)
        .await
        .map_err(AppError::from)?;

    queries::exclusion_zones::delete_area(&mut *tx, row.area_id)
        .await
        .map_err(AppError::from)?;

    if let Some(line_id) = row.line_id {
        queries::exclusion_zones::delete_line(&mut *tx, line_id)
            .await
            .map_err(AppError::from)?;
    } else if let Some(circle_id) = row.circle_id {
        queries::exclusion_zones::delete_circle(&mut *tx, circle_id)
            .await
            .map_err(AppError::from)?;
    } else if let Some(polygon_id) = row.polygon_id {
        queries::exclusion_zones::delete_polygon(&mut *tx, polygon_id)
            .await
            .map_err(AppError::from)?;
    }

    tx.commit().await.map_err(AppError::from)?;

    Ok(())
}

async fn build_area(
    row: &queries::exclusion_zones::ExclusionZoneRow,
    pool: &PgPool,
) -> Result<Area, AppError> {
    if let Some(polygon_id) = row.polygon_id {
        let points = queries::exclusion_zones::get_polygon_points(pool, polygon_id)
            .await
            .map_err(AppError::from)?;
        Ok(Area::Polygon(Polygon { vertices: points }))
    } else if row.line_id.is_some() {
        Ok(Area::Line(Line {
            start: Point {
                lat: row.start_lat.unwrap(),
                lng: row.start_lng.unwrap(),
            },
            end: Point {
                lat: row.end_lat.unwrap(),
                lng: row.end_lng.unwrap(),
            },
        }))
    } else {
        Ok(Area::Circle(Circle {
            center: Point {
                lat: row.center_lat.unwrap(),
                lng: row.center_lng.unwrap(),
            },
            radius: row.radius_meters.unwrap() as f64,
        }))
    }
}
