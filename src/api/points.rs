use actix_web::{post, web, HttpResponse};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct NewPoint {
    pub randomized_id: i64,
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    pub spd: f64,
    pub azm: f64,
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

#[post("/")]
pub async fn push_points (
    db: web::Data<DatabaseConnection>,
    req: web::Json<PointListRequest>,
) -> HttpResponse {
    let points = req.into_inner().points;

    for point in points {
        let new_point = crate::database::model::points::ActiveModel {
            randomized_id: Set(point.randomized_id),
            lat: Set(point.lat),
            lon: Set(point.lon),
            alt: Set(point.alt),
            spd: Set(point.spd),
            azm: Set(point.azm),
            ..Default::default()
        };

        if let Err(e) = new_point.insert(db.get_ref()).await {
            eprintln!("Failed to insert point: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Ok().finish()
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/points")
            .service(push_points)
    );
}