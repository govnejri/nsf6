use actix_web::{get, web, HttpResponse};
use chrono::DateTime;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use log::{info, warn, error, debug};
use std::time::Instant;
use sea_orm::QueryOrder;
use crate::database::model::points::{self, Entity as Points};

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct MapPoint {
    pub lat: f64,
    pub long: f64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct MapRectangle {
    pub top_left: MapPoint,
    pub bottom_right: MapPoint,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SpeedmapRequest {
    pub area: MapRectangle,
    pub time_start: DateTime<chrono::Utc>,
    pub time_end: DateTime<chrono::Utc>,
    pub tile_width: f64,
    pub tile_height: f64,
}

// Flat query parameters for GET requests (external names in camelCase)
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SpeedmapQueryParams {
    /// Top-left latitude
    #[serde(rename = "tlLat")]
    pub tl_lat: f64,
    /// Top-left longitude
    #[serde(rename = "tlLong")]
    pub tl_long: f64,
    /// Bottom-right latitude
    #[serde(rename = "brLat")]
    pub br_lat: f64,
    /// Bottom-right longitude
    #[serde(rename = "brLong")]
    pub br_long: f64,
    #[serde(rename = "timeStart")]
    pub time_start: DateTime<chrono::Utc>,
    #[serde(rename = "timeEnd")]
    pub time_end: DateTime<chrono::Utc>,
    #[serde(rename = "tileWidth")]
    pub tile_width: f64,
    #[serde(rename = "tileHeight")]
    pub tile_height: f64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct SpeedTile {
    #[serde(rename = "avgVelocity")]
    pub avg_velocity: f64,
    #[serde(rename = "neighborAvgVelocity")]
    pub neighbor_avg_velocity: f64,
    #[serde(rename = "topLeft")]
    pub top_left: MapPoint,
    #[serde(rename = "bottomRight")]
    pub bottom_right: MapPoint,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct SpeedmapData {
    pub data: Vec<SpeedTile>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct SpeedmapResponse {
    pub speedmap: SpeedmapData,
}

#[utoipa::path(
    get,
    path = "/api/speedmap",
    tag = "Speedmap",
    params(
    ("tlLat" = f64, Query, description = "Top-left latitude"),
    ("tlLong" = f64, Query, description = "Top-left longitude"),
    ("brLat" = f64, Query, description = "Bottom-right latitude"),
    ("brLong" = f64, Query, description = "Bottom-right longitude"),
    ("timeStart" = DateTime<chrono::Utc>, Query, description = "Start of the time range (inclusive)"),
    ("timeEnd" = DateTime<chrono::Utc>, Query, description = "End of the time range (inclusive)"),
    ("tileWidth" = f64, Query, description = "Width of each tile in degrees"),
    ("tileHeight" = f64, Query, description = "Height of each tile in degrees"),
    ),
    responses(
        (status = 200, description = "Speedmap data", body = SpeedmapResponse),
        (status = 500, description = "Server Vzorvalsya"),
    )
)]

#[get("")]
pub async fn get_speedmap(
    db: web::Data<DatabaseConnection>,
    qp: web::Query<SpeedmapQueryParams>,
) -> HttpResponse {
    let started = Instant::now();
    debug!(
        "Speedmap request: tl=({}, {}), br=({}, {}), time=[{}..{}], tile=({}, {})",
        qp.tl_lat, qp.tl_long, qp.br_lat, qp.br_long, qp.time_start, qp.time_end, qp.tile_width, qp.tile_height
    );
    // Basic validation
    if qp.tile_width <= 0.0 || qp.tile_height <= 0.0 {
        warn!("Invalid tile size: width={}, height={}", qp.tile_width, qp.tile_height);
        return HttpResponse::BadRequest().body("tileWidth and tileHeight must be > 0");
    }

    // Allow any two opposite corners; compute bounds
    let (lat_min, lat_max) = if qp.tl_lat <= qp.br_lat { (qp.tl_lat, qp.br_lat) } else { (qp.br_lat, qp.tl_lat) };
    let (lon_min, lon_max) = if qp.tl_long <= qp.br_long { (qp.tl_long, qp.br_long) } else { (qp.br_long, qp.tl_long) };

    let lat_span = (lat_max - lat_min).max(0.0);
    let lon_span = (lon_max - lon_min).max(0.0);

    let rows = if lat_span == 0.0 { 0 } else { ((lat_span / qp.tile_height).ceil() as usize).max(1) };
    let cols = if lon_span == 0.0 { 0 } else { ((lon_span / qp.tile_width).ceil() as usize).max(1) };

    // Early return if degenerate
    if rows == 0 || cols == 0 {
        let resp = SpeedmapResponse { speedmap: SpeedmapData { data: vec![] } };
    info!("Speedmap degenerate area (rows=0 or cols=0), returning empty. took={:?}", started.elapsed());
        return HttpResponse::Ok().json(resp);
    }

    // First, get all points within bounds and time range, ordered by timestamp
    let all_points = match Points::find()
        .filter(points::Column::Lat.between(lat_min, lat_max))
        .filter(points::Column::Lon.between(lon_min, lon_max))
        .filter(points::Column::Timestamp.gte(qp.time_start))
        .filter(points::Column::Timestamp.lte(qp.time_end))
        .order_by_asc(points::Column::Timestamp)
        .all(db.get_ref()).await {
        Ok(p) => p,
        Err(e) => {
            error!("Speedmap query failed: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Use all points without deduplication
    let total_points_count = all_points.len();
    debug!(
        "Speedmap DB returned {} points in {:?}",
        total_points_count,
        started.elapsed()
    );

    // Bucket points into tiles (sum of speeds and counts for averages)
    let mut counts = vec![0usize; rows * cols];
    let mut speed_sums = vec![0f64; rows * cols];
    let inv_h = 1.0 / qp.tile_height;
    let inv_w = 1.0 / qp.tile_width;

    for p in all_points {
        // Compute indices; clamp to [0, rows-1] / [0, cols-1]
        let mut r = ((p.lat - lat_min) * inv_h).floor() as isize;
        let mut c = ((p.lon - lon_min) * inv_w).floor() as isize;

        if r < 0 { r = 0; }
        if c < 0 { c = 0; }
        if r as usize >= rows { r = rows as isize - 1; }
        if c as usize >= cols { c = cols as isize - 1; }

        let idx = (r as usize) * cols + (c as usize);
        counts[idx] += 1;
        // use p.spd as velocity contribution
        speed_sums[idx] += p.spd;
    }

    // Build response tiles (row-major from lat_min/lon_min increasing)
    // Include tiles with data if tile has points or neighbors have points
    let mut data = Vec::new();
    for r in 0..rows {
        let tile_lat_min = lat_min + (r as f64) * qp.tile_height;
        let tile_lat_max = (tile_lat_min + qp.tile_height).min(lat_max);
        for c in 0..cols {
            let tile_lon_min = lon_min + (c as f64) * qp.tile_width;
            let tile_lon_max = (tile_lon_min + qp.tile_width).min(lon_max);

            let idx = r * cols + c;
            let count = counts[idx];
            let sum = speed_sums[idx];
            let avg_velocity = if count > 0 { sum / (count as f64) } else { 0.0 };

            // Calculate neighbor average velocity (8 surrounding cells)
            let mut neighbor_sum = 0.0f64;
            let mut neighbor_points = 0usize;
            for dr in -1..=1 {
                for dc in -1..=1 {
                    // Skip the center cell (the current tile itself)
                    if dr == 0 && dc == 0 {
                        continue;
                    }

                    let nr = r as isize + dr;
                    let nc = c as isize + dc;

                    // Check bounds
                    if nr >= 0 && nr < rows as isize && nc >= 0 && nc < cols as isize {
                        let neighbor_idx = (nr as usize) * cols + (nc as usize);
                        neighbor_sum += speed_sums[neighbor_idx];
                        neighbor_points += counts[neighbor_idx];
                    }
                }
            }

            let neighbor_avg_velocity = if neighbor_points > 0 { neighbor_sum / (neighbor_points as f64) } else { 0.0 };

            // Include tiles with own data or neighbor data
            if count > 0 || neighbor_points > 0 {
                data.push(SpeedTile {
                    avg_velocity,
                    neighbor_avg_velocity,
                    top_left: MapPoint { lat: tile_lat_min, long: tile_lon_min },
                    bottom_right: MapPoint { lat: tile_lat_max, long: tile_lon_max },
                });
            }
        }
    }

    let resp = SpeedmapResponse { speedmap: SpeedmapData { data } };
    info!(
        "Speedmap response: tiles={} (non-zero only) from grid={}x{} total_points={} took={:?}",
        resp.speedmap.data.len(), rows, cols, counts.iter().sum::<usize>(), started.elapsed()
    );
    HttpResponse::Ok().json(resp)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/speedmap")
            .service(get_speedmap)
    );
}