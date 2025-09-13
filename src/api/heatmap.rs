use actix_web::{get, web, HttpResponse};
use chrono::DateTime;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
pub struct HeatmapRequest {
    pub area: MapRectangle,
    pub time_start: DateTime<chrono::Utc>,
    pub time_end: DateTime<chrono::Utc>,
    pub tile_width: f64,
    pub tile_height: f64,
}

// Flat query parameters for GET requests
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct HeatmapQueryParams {
    /// First point (any corner) latitude
    pub lat1: f64,
    /// First point (any corner) longitude
    pub lon1: f64,
    /// Second point (opposite corner) latitude
    pub lat2: f64,
    /// Second point (opposite corner) longitude
    pub lon2: f64,
    pub time_start: DateTime<chrono::Utc>,
    pub time_end: DateTime<chrono::Utc>,
    pub tile_width: f64,
    pub tile_height: f64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct HeatTile {
    pub count: usize,
    #[serde(rename = "topLeft")]
    pub top_left: MapPoint,
    #[serde(rename = "bottomRight")]
    pub bottom_right: MapPoint,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct HeatmapData {
    pub data: Vec<HeatTile>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct HeatmapResponse {
    pub heatmap: HeatmapData,
}

#[utoipa::path(
    get,
    path = "/api/heatmap",
    tag = "Heatmap",
    params(
        ("lat1" = f64, Query, description = "Corner 1 latitude"),
        ("lon1" = f64, Query, description = "Corner 1 longitude"),
        ("lat2" = f64, Query, description = "Corner 2 latitude"),
        ("lon2" = f64, Query, description = "Corner 2 longitude"),
        ("time_start" = DateTime<chrono::Utc>, Query, description = "Start of the time range (inclusive)"),
        ("time_end" = DateTime<chrono::Utc>, Query, description = "End of the time range (inclusive)"),
        ("tile_width" = f64, Query, description = "Width of each tile in degrees"),
        ("tile_height" = f64, Query, description = "Height of each tile in degrees"),
    ),
    responses(
        (status = 200, description = "Heatmap data", body = HeatmapResponse),
        (status = 500, description = "Server Vzorvalsya"),
    )
)]

#[get("")]
pub async fn get_heatmap(
    db: web::Data<DatabaseConnection>,
    qp: web::Query<HeatmapQueryParams>,
) -> HttpResponse {
    // Basic validation
    if qp.tile_width <= 0.0 || qp.tile_height <= 0.0 {
        return HttpResponse::BadRequest().body("tile_width and tile_height must be > 0");
    }

    // Allow any two opposite corners; compute bounds
    let (lat_min, lat_max) = if qp.lat1 <= qp.lat2 { (qp.lat1, qp.lat2) } else { (qp.lat2, qp.lat1) };
    let (lon_min, lon_max) = if qp.lon1 <= qp.lon2 { (qp.lon1, qp.lon2) } else { (qp.lon2, qp.lon1) };

    let lat_span = (lat_max - lat_min).max(0.0);
    let lon_span = (lon_max - lon_min).max(0.0);

    let rows = if lat_span == 0.0 { 0 } else { ((lat_span / qp.tile_height).ceil() as usize).max(1) };
    let cols = if lon_span == 0.0 { 0 } else { ((lon_span / qp.tile_width).ceil() as usize).max(1) };

    // Early return if degenerate
    if rows == 0 || cols == 0 {
        let resp = HeatmapResponse { heatmap: HeatmapData { data: vec![] } };
        return HttpResponse::Ok().json(resp);
    }

    // Query points within bounds and time range
    use crate::database::model::points::{self, Entity as Points};

    let query = Points::find()
        .filter(points::Column::Lat.between(lat_min, lat_max))
        .filter(points::Column::Lon.between(lon_min, lon_max))
        .filter(points::Column::Timestamp.gte(qp.time_start))
        .filter(points::Column::Timestamp.lte(qp.time_end));

    let points = match query.all(db.get_ref()).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Heatmap query failed: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Bucket points into tiles
    let mut counts = vec![0usize; rows * cols];
    let inv_h = 1.0 / qp.tile_height;
    let inv_w = 1.0 / qp.tile_width;

    for p in points {
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
    let mut data = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        let tile_lat_min = lat_min + (r as f64) * qp.tile_height;
        let tile_lat_max = (tile_lat_min + qp.tile_height).min(lat_max);
        for c in 0..cols {
            let tile_lon_min = lon_min + (c as f64) * qp.tile_width;
            let tile_lon_max = (tile_lon_min + qp.tile_width).min(lon_max);

            let count = counts[r * cols + c];
            data.push(HeatTile {
                count,
                top_left: MapPoint { lat: tile_lat_min, long: tile_lon_min },
                bottom_right: MapPoint { lat: tile_lat_max, long: tile_lon_max },
            });
        }
    }

    let resp = HeatmapResponse { heatmap: HeatmapData { data } };
    HttpResponse::Ok().json(resp)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/heatmap")
            .service(get_heatmap)
    );
}