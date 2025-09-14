use actix_web::{HttpResponse, Error};
use minijinja::context;

pub async fn index() -> Result<HttpResponse, Error> {
    crate::templates::render_template(
        "index",
        context! {},
    )
}