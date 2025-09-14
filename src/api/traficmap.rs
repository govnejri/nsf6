use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, NaiveTime, Weekday, Datelike};
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
pub struct TraficmapRequest {
    pub area: MapRectangle,
    pub time_start: DateTime<chrono::Utc>,
    pub time_end: DateTime<chrono::Utc>,
    pub tile_width: f64,
    pub tile_height: f64,
}

// Flat query parameters for GET requests (external names in camelCase)
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TraficmapQueryParams {
    /// First latitude (corner)
    #[serde(rename = "lat1")]
    pub lat1: f64,
    /// First longitude (corner)
    #[serde(rename = "lng1")]
    pub lng1: f64,
    /// Second latitude (opposite corner)
    #[serde(rename = "lat2")]
    pub lat2: f64,
    /// Second longitude (opposite corner)
    #[serde(rename = "lng2")]
    pub lng2: f64,
    /// Optional date range start (inclusive)
    #[serde(rename = "dateStart")]
    pub date_start: Option<DateTime<chrono::Utc>>,
    /// Optional date range end (inclusive)
    #[serde(rename = "dateEnd")]
    pub date_end: Option<DateTime<chrono::Utc>>,
    #[serde(rename = "tileWidth")]
    pub tile_width: f64,
    #[serde(rename = "tileHeight")]
    pub tile_height: f64,
    /// Optional list of weekdays 1..7, comma/space separated
    #[serde(rename = "days")]
    pub days: Option<String>,
    /// Optional time-of-day start in HH or HH:MM (inclusive)
    #[serde(rename = "timeStart")]
    pub time_start_tod: Option<String>,
    /// Optional time-of-day end in HH or HH:MM (exclusive)
    #[serde(rename = "timeEnd")]
    pub time_end_tod: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct TraficTile {
    pub count: usize,
    #[serde(rename = "neighborCount")]
    pub neighbor_count: usize,
    #[serde(rename = "topLeft")]
    pub top_left: MapPoint,
    #[serde(rename = "bottomRight")]
    pub bottom_right: MapPoint,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct TraficmapData {
    pub data: Vec<TraficTile>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct TraficmapResponse {
    pub traficmap: TraficmapData,
}

#[utoipa::path(
    get,
    path = "/api/traficmap",
    tag = "Traficmap",
    params(
    ("lat1" = f64, Query, description = "First latitude (corner)"),
    ("lng1" = f64, Query, description = "First longitude (corner)"),
    ("lat2" = f64, Query, description = "Second latitude (opposite corner)"),
    ("lng2" = f64, Query, description = "Second longitude (opposite corner)"),
    ("dateStart" = DateTime<chrono::Utc>, Query, description = "Start of the date/time range (inclusive). Optional"),
    ("dateEnd" = DateTime<chrono::Utc>, Query, description = "End of the date/time range (inclusive). Optional"),
    ("tileWidth" = f64, Query, description = "Width of each tile in degrees"),
    ("tileHeight" = f64, Query, description = "Height of each tile in degrees"),
    ("days" = String, Query, description = "Optional list of weekdays to include (1=Mon..7=Sun). Comma or space separated"),
    ("timeStart" = String, Query, description = "Optional time-of-day start in HH or HH:MM (inclusive)"),
    ("timeEnd" = String, Query, description = "Optional time-of-day end in HH or HH:MM (exclusive)"),
    ),
    responses(
        (status = 200, description = "Traficmap data", body = TraficmapResponse),
        (status = 500, description = "Server Vzorvalsya"),
    )
)]

#[get("")]
pub async fn get_traficmap(
    db: web::Data<DatabaseConnection>,
    qp: web::Query<TraficmapQueryParams>,
) -> HttpResponse {
    let started = Instant::now();
    debug!(
        "Traficmap request: corners=({}, {}), ({}, {}), date=[{:?}..{:?}], tile=({}, {}), days={:?}, tod=[{:?}..{:?}]",
        qp.lat1, qp.lng1, qp.lat2, qp.lng2, qp.date_start, qp.date_end, qp.tile_width, qp.tile_height, qp.days, qp.time_start_tod, qp.time_end_tod
    );
    // Basic validation
    if qp.tile_width <= 0.0 || qp.tile_height <= 0.0 {
        warn!("Invalid tile size: width={}, height={}", qp.tile_width, qp.tile_height);
        return HttpResponse::BadRequest().body("tileWidth and tileHeight must be > 0");
    }

    // Allow any two opposite corners; compute bounds
    let (lat_min, lat_max) = if qp.lat1 <= qp.lat2 { (qp.lat1, qp.lat2) } else { (qp.lat2, qp.lat1) };
    let (lon_min, lon_max) = if qp.lng1 <= qp.lng2 { (qp.lng1, qp.lng2) } else { (qp.lng2, qp.lng1) };

    let lat_span = (lat_max - lat_min).max(0.0);
    let lon_span = (lon_max - lon_min).max(0.0);

    let rows = if lat_span == 0.0 { 0 } else { ((lat_span / qp.tile_height).ceil() as usize).max(1) };
    let cols = if lon_span == 0.0 { 0 } else { ((lon_span / qp.tile_width).ceil() as usize).max(1) };

    // Early return if degenerate
    if rows == 0 || cols == 0 {
        let resp = TraficmapResponse { traficmap: TraficmapData { data: vec![] } };
    info!("Traficmap degenerate area (rows=0 or cols=0), returning empty. took={:?}", started.elapsed());
        return HttpResponse::Ok().json(resp);
    }

    // First, get all points within bounds and optional time range, ordered by timestamp
    let mut query = Points::find()
        .filter(points::Column::Lat.between(lat_min, lat_max))
        .filter(points::Column::Lon.between(lon_min, lon_max));
    if let Some(ts_start) = qp.date_start { query = query.filter(points::Column::Timestamp.gte(ts_start)); }
    if let Some(ts_end) = qp.date_end { query = query.filter(points::Column::Timestamp.lte(ts_end)); }
    let mut all_points = match query
        .order_by_asc(points::Column::Timestamp)
        .all(db.get_ref()).await {
        Ok(p) => p,
        Err(e) => {
            error!("Traficmap query failed: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Apply optional weekday and time-of-day filters
    let day_set = match &qp.days {
        Some(s) => match parse_days_of_week(s) { Ok(set) => Some(set), Err(e) => {
            warn!("Invalid days parameter '{}': {}", s, e);
            return HttpResponse::BadRequest().body("days must contain numbers 1..7 separated by comma/space");
        }},
        None => None,
    };
    let (tod_start, tod_end) = match (&qp.time_start_tod, &qp.time_end_tod) {
        (Some(a), Some(b)) => {
            let a = match parse_time_of_day(a) { Ok(t) => t, Err(_) => { return HttpResponse::BadRequest().body("timeStart must be HH or HH:MM"); }};
            let b = match parse_time_of_day(b) { Ok(t) => t, Err(_) => { return HttpResponse::BadRequest().body("timeEnd must be HH or HH:MM"); }};
            if b <= a { warn!("Invalid time-of-day window: start={:?} end={:?}", a, b); return HttpResponse::BadRequest().body("timeEnd must be greater than timeStart (same-day window)"); }
            (Some(a), Some(b))
        }
        (None, None) => (None, None),
        _ => { return HttpResponse::BadRequest().body("Both timeStart and timeEnd must be provided together"); }
    };
    if day_set.is_some() || tod_start.is_some() {
        all_points = all_points.into_iter().filter(|p| {
            if let Some(ref set) = day_set {
                if let Some(ts) = p.timestamp { let wd = ts.weekday(); let day_num = match wd { Weekday::Mon=>1,Weekday::Tue=>2,Weekday::Wed=>3,Weekday::Thu=>4,Weekday::Fri=>5,Weekday::Sat=>6,Weekday::Sun=>7 }; if !set.contains(&day_num) { return false; } } else { return false; }
            }
            match (tod_start, tod_end) { (Some(s), Some(e)) => { if let Some(ts) = p.timestamp { let t = ts.time(); t >= s && t < e } else { false } } _ => true }
        }).collect();
    }
    let total_points_count = all_points.len();
    debug!("Traficmap DB returned {} points after filters in {:?}", total_points_count, started.elapsed());

    // Bucket points into tiles
    let mut counts = vec![0usize; rows * cols];
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
    }

    // Build response tiles (row-major from lat_min/lon_min increasing)
    // Include tiles with count > 0 OR neighbor_count > 0
    let mut data = Vec::new();
    for r in 0..rows {
        let tile_lat_min = lat_min + (r as f64) * qp.tile_height;
        let tile_lat_max = (tile_lat_min + qp.tile_height).min(lat_max);
        for c in 0..cols {
            let tile_lon_min = lon_min + (c as f64) * qp.tile_width;
            let tile_lon_max = (tile_lon_min + qp.tile_width).min(lon_max);

            let count = counts[r * cols + c];
            // Calculate neighbor count (8 surrounding cells)
            let mut neighbor_count = 0;
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
                        neighbor_count += counts[neighbor_idx];
                    }
                }
            }

            // Include tiles with points or with non-zero neighbors
            if count > 0 || neighbor_count > 0 {
                data.push(TraficTile {
                    count,
                    neighbor_count,
                    top_left: MapPoint { lat: tile_lat_min, long: tile_lon_min },
                    bottom_right: MapPoint { lat: tile_lat_max, long: tile_lon_max },
                });
            }
        }
    }

    let resp = TraficmapResponse { traficmap: TraficmapData { data } };
    info!(
        "Traficmap response: tiles={} (non-zero only) from grid={}x{} points_count={} took={:?}",
        resp.traficmap.data.len(), rows, cols, counts.iter().sum::<usize>(), started.elapsed()
    );
    HttpResponse::Ok().json(resp)
}

// --- Helpers ---

fn parse_days_of_week(input: &str) -> Result<std::collections::HashSet<u8>, String> {
    let mut set = std::collections::HashSet::new();
    for token in input.split(|c: char| c == ',' || c.is_whitespace()) {
        let t = token.trim();
        if t.is_empty() { continue; }
        let n: u8 = t.parse().map_err(|_| format!("invalid day '{}': not a number", t))?;
        if n == 0 || n > 7 { return Err(format!("day '{}' out of range 1..7", n)); }
        set.insert(n);
    }
    if set.is_empty() { return Err("no valid days provided".to_string()); }
    Ok(set)
}

fn parse_time_of_day(input: &str) -> Result<NaiveTime, String> {
    let s = input.trim();
    if let Ok(t) = NaiveTime::parse_from_str(s, "%H:%M") { return Ok(t); }
    if let Ok(h) = s.parse::<u32>() { return Ok(NaiveTime::from_hms_opt(h, 0, 0).ok_or("hour out of range")?); }
    if let Ok(t) = NaiveTime::parse_from_str(s, "%H:%M:%S") { return Ok(t); }
    Err("invalid time format".to_string())
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/trafficmap")
            .service(get_traficmap)
    );
}