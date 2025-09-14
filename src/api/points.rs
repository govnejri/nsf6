use actix_web::{post, web, HttpResponse};
use sea_orm::{DatabaseConnection, Set, EntityTrait, ColumnTrait, QueryOrder, QueryFilter, ActiveModelTrait};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use log::{info, warn, error};
use std::time::Instant;
use chrono::{DateTime, Utc};
use std::env;

use crate::database::model::points::{Entity as Points, Column as PointsColumn, Model as PointModel, ActiveModel as PointActiveModel};

#[derive(Debug, Serialize, Deserialize)]
struct WebhookPoint {
    lat: f64,
    lng: f64,
    azm: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookPayload {
    first: WebhookPoint,
    second: WebhookPoint,
    gone: Vec<WebhookPoint>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct NewPoint {
    pub randomized_id: i64,
    pub lat: f64,
    pub lng: f64,
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

    // Resolve webhook URL from env; if missing, we still insert without webhook/anomaly
    let webhook_url = env::var("POINTS_WEBHOOK_URL").ok();

    // Process points one-by-one to follow the described pipeline
    for p in points {
        // Build ActiveModel with defaults
        let mut active = PointActiveModel {
            randomized_id: Set(p.randomized_id),
            lat: Set(p.lat),
            lng: Set(p.lng),
            alt: Set(p.alt.unwrap_or(0.0)),
            spd: Set(p.spd),
            azm: Set(p.azm),
            ..Default::default()
        };

        // Only set timestamp if provided; otherwise, leave NotSet to use DB default
        if let Some(ts) = p.timestamp {
            active.timestamp = Set(Some(ts));
        }

        let mut anomaly_value: Option<bool> = None;

        if let Some(url) = &webhook_url {
            // Query existing points with same randomized_id
            match Points::find()
                .filter(PointsColumn::RandomizedId.eq(p.randomized_id))
                .order_by_desc(PointsColumn::Timestamp)
                .all(db.get_ref())
                .await
            {
                Ok(existing) => {
                    if existing.is_empty() {
                        // Case 1: no existing points -> just insert (no webhook)
                    } else {
                        // Build payload according to rules
                        let second_ts = p.timestamp.unwrap_or_else(|| Utc::now());
                        let second = WebhookPoint { lat: p.lat, lng: p.lng, azm: p.azm, timestamp: second_ts };

                        // First is either the only one or the most recent from DB
                        let first_model: &PointModel = &existing[0];
                        // Convert DB model to webhook point; fallback timestamp to now if missing
                        let first_ts = first_model.timestamp.unwrap_or_else(|| Utc::now());
                        let first = WebhookPoint { lat: first_model.lat, lng: first_model.lng, azm: first_model.azm, timestamp: first_ts };

                        // Gone: rest of DB points (skip first), by descending timestamp
                        let mut gone: Vec<WebhookPoint> = Vec::new();
                        if existing.len() > 1 {
                            for m in existing.iter().skip(1) {
                                let ts = m.timestamp.unwrap_or_else(|| Utc::now());
                                gone.push(WebhookPoint { lat: m.lat, lng: m.lng, azm: m.azm, timestamp: ts });
                            }
                        }

                        let payload = WebhookPayload { first, second, gone };

                        // Send POST
                        let client = reqwest::Client::new();
                        match client.post(url).json(&payload).send().await {
                            Ok(resp) => {
                                // Read response body as text and try to parse into i32 either as JSON or plain text
                                let code_opt: Option<i32> = match resp.text().await {
                                    Ok(body) => {
                                        serde_json::from_str::<i32>(&body).ok()
                                            .or_else(|| body.trim().parse::<i32>().ok())
                                    }
                                    Err(_) => None,
                                };

                                match code_opt {
                                    Some(-1) => anomaly_value = Some(true),
                                    Some(1) => anomaly_value = Some(false),
                                    Some(other) => {
                                        warn!("Unexpected webhook response code: {}", other);
                                    }
                                    None => {
                                        warn!("Failed to parse webhook response for rid {}", p.randomized_id);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Webhook POST failed: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("DB query failed for rid {}: {}", p.randomized_id, e);
                }
            }
        } else {
            // No webhook configured
            warn!("POINTS_WEBHOOK_URL is not set; skipping webhook calls");
        }

        // Set anomaly if determined
        if anomaly_value.is_some() {
            active.anomaly = Set(anomaly_value);
        }

        // Insert the point
        if let Err(e) = active.insert(db.get_ref()).await {
            error!("Insert failed for rid {}: {}", p.randomized_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    info!("Processed and inserted points in {:?}", started.elapsed());
    HttpResponse::Ok().finish()
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/points")
            .service(push_points)
    );
}