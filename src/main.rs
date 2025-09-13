use actix_files as fs;
use actix_web::{App, HttpServer, web};
mod routes;
mod templates;
mod image_compressor;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Compress::default())
            // Оптимизированный обработчик для изображений
            .route("/static/assets/img/{filename:.*}", web::get().to(image_compressor::serve_optimized_image))
            // Обычные статические файлы
            .service(
                fs::Files::new("/static", "web/out/static")
                    .prefer_utf8(true)
                    .use_etag(true)
                    .use_last_modified(true)
            )
            .route("/", web::get().to(routes::index))
            .default_service(web::route().to(routes::not_found))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
