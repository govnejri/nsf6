use actix_web::{HttpResponse, Error, HttpRequest};
use minijinja::context;

pub async fn map(_req: HttpRequest) -> Result<HttpResponse, Error> {
    crate::templates::render_template(
        "map",
        context! {},
    )
}
