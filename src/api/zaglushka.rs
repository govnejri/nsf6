use actix_web::{post, web, HttpResponse};

// Temporary stub endpoint: responds with integer 1 to any POST payload
#[post("")]
pub async fn stub_always_one(_body: web::Bytes) -> HttpResponse {
    HttpResponse::Ok().json(1)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/zaglushka")
            .service(stub_always_one)
    );
}
