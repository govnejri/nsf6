use actix_files as fs;
use actix_web::{web, App, HttpServer, middleware};
mod routes;
mod templates;
mod image_compressor;
mod database;
mod api;
use api::{points, heatmap};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Compress::default())
            .route("/static/assets/img/{filename:.*}", web::get().to(image_compressor::serve_optimized_image))
            .service(
                fs::Files::new("/static", "web/out/static")
                    .prefer_utf8(true)
                    .use_etag(true)
                    .use_last_modified(true)
            )
            .route("/", web::get().to(routes::index))
            .service(web::scope("/api")
                .wrap(middleware::NormalizePath::trim())
                .configure(points::init_routes)
                .configure(heatmap::init_routes)
            )
            .default_service(web::route().to(routes::not_found))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
