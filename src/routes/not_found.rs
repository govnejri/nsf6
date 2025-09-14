use actix_web::{HttpResponse, Error, HttpRequest};
use minijinja::context;

pub async fn not_found(_req: HttpRequest) -> Result<HttpResponse, Error> {
    crate::templates::render_template(
        "404",
        context! {
        },
    )
}
