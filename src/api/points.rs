use actix_web::{post, web, HttpResponse};
use sea_orm::{DatabaseConnection, Set, EntityTrait};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use log::{info, error};
use std::time::Instant;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct NewPoint {
    pub randomized_id: i64,
    pub lat: f64,
    pub lon: f64,
    /// Optional altitude; defaults to 0 if not provided
    pub alt: Option<f64>,
    pub spd: f64,
    pub azm: f64,
    /// Optional timestamp in RFC3339/ISO8601 with timezone, e.g. "2025-09-14T12:34:56+06:00"
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PointListRequest {
    pub points: Vec<NewPoint>,
}

#[utoipa::path(
    post,
    path = "/api/points",
    tag = "Points",
    
    responses(
        (status = 200, description = "List of points", body = PointListRequest),
        (status = 500, description = "Incorrect point list format")
    )
)]

#[post("")]
pub async fn push_points (
    db: web::Data<DatabaseConnection>,
    req: web::Json<PointListRequest>,
) -> HttpResponse {
    let started = Instant::now();
    let points = req.into_inner().points;
    info!("Received {} points to insert", points.len());

    if points.is_empty() {
        return HttpResponse::BadRequest().body("Empty points list");
    }

    let models: Vec<crate::database::model::points::ActiveModel> = points
        .into_iter()
        .map(|point| {
            let mut model = crate::database::model::points::ActiveModel {
                randomized_id: Set(point.randomized_id),
                lat: Set(point.lat),
                lon: Set(point.lon),
                alt: Set(point.alt.unwrap_or(0.0)),
                spd: Set(point.spd),
                azm: Set(point.azm),
                ..Default::default()
            };

            // Only set timestamp if provided; otherwise, leave NotSet to use DB default
            if let Some(ts) = point.timestamp {
                model.timestamp = Set(Some(ts));
            }

            model
        })
        .collect();

    match crate::database::model::points::Entity::insert_many(models).exec(db.get_ref()).await {
        Ok(_) => {
            info!("Inserted points in {:?}", started.elapsed());
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            error!("Batch insert failed: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/points")
            .service(push_points)
    );
}