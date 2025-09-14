use actix_web::{get, web, HttpResponse};
use chrono::DateTime;
use log::{debug, error};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::database::model::points::{self, Entity as Points};

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct MapPointTs {
	pub lat: f64,
	pub lng: f64,
	pub timestamp: Option<DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct AnomalyRoute {
	pub randomized_id: i64,
	pub points: Vec<MapPointTs>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct AnomaliesResponse {
	pub anomalies: Vec<AnomalyRoute>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AnomaliesQueryParams {
	#[serde(rename = "lat1")] pub lat1: f64,
	#[serde(rename = "lng1")] pub lng1: f64,
	#[serde(rename = "lat2")] pub lat2: f64,
	#[serde(rename = "lng2")] pub lng2: f64,
	#[serde(rename = "dateStart")] pub date_start: Option<DateTime<chrono::Utc>>, // inclusive
	#[serde(rename = "dateEnd")] pub date_end: Option<DateTime<chrono::Utc>>,     // inclusive
}

#[utoipa::path(
	get,
	path = "/api/anomalies",
	tag = "Anomalies",
	params(
		("lat1" = f64, Query, description = "First latitude (corner)"),
		("lng1" = f64, Query, description = "First longitude (corner)"),
		("lat2" = f64, Query, description = "Second latitude (opposite corner)"),
		("lng2" = f64, Query, description = "Second longitude (opposite corner)"),
		("dateStart" = DateTime<chrono::Utc>, Query, description = "Start of the date/time range (inclusive). Optional"),
		("dateEnd" = DateTime<chrono::Utc>, Query, description = "End of the date/time range (inclusive). Optional"),
	),
	responses(
		(status = 200, description = "Anomalous routes", body = AnomaliesResponse),
		(status = 500, description = "Server error"),
	)
)]
#[get("")]
pub async fn get_anomalies(
	db: web::Data<DatabaseConnection>,
	qp: web::Query<AnomaliesQueryParams>,
) -> HttpResponse {
	let (lat_min, lat_max) = if qp.lat1 <= qp.lat2 { (qp.lat1, qp.lat2) } else { (qp.lat2, qp.lat1) };
	let (lng_min, lng_max) = if qp.lng1 <= qp.lng2 { (qp.lng1, qp.lng2) } else { (qp.lng2, qp.lng1) };

	let mut query = Points::find()
		.filter(points::Column::Lat.between(lat_min, lat_max))
		.filter(points::Column::Lng.between(lng_min, lng_max))
		.filter(points::Column::Anomaly.eq(Some(true)));

	if let Some(start) = qp.date_start {
		query = query.filter(points::Column::Timestamp.gte(start));
	}
	if let Some(end) = qp.date_end {
		query = query.filter(points::Column::Timestamp.lte(end));
	}

	let rows = match query
		.order_by_asc(points::Column::RandomizedId)
		.order_by_asc(points::Column::Timestamp)
		.all(db.get_ref())
		.await
	{
		Ok(r) => r,
		Err(e) => {
			error!("Anomalies query failed: {}", e);
			return HttpResponse::InternalServerError().finish();
		}
	};

	// Group rows by randomized_id into routes
	let mut routes: Vec<AnomalyRoute> = Vec::new();
	let mut cur_id: Option<i64> = None;
	let mut cur_points: Vec<MapPointTs> = Vec::new();

	for row in rows.into_iter() {
		if cur_id != Some(row.randomized_id) {
			if let Some(id) = cur_id {
				routes.push(AnomalyRoute { randomized_id: id, points: cur_points });
				cur_points = Vec::new();
			}
			cur_id = Some(row.randomized_id);
		}
		cur_points.push(MapPointTs { lat: row.lat, lng: row.lng, timestamp: row.timestamp });
	}
	if let Some(id) = cur_id {
		routes.push(AnomalyRoute { randomized_id: id, points: cur_points });
	}

	debug!(
		"Anomalies response: routes={} points_total={}",
		routes.len(),
		routes.iter().map(|r| r.points.len()).sum::<usize>()
	);
	HttpResponse::Ok().json(AnomaliesResponse { anomalies: routes })
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(web::scope("/anomalies").service(get_anomalies));
}
