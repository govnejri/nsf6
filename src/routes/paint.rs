use actix_web::{HttpResponse, Error};
use minijinja::context;

pub async fn paint() -> Result<HttpResponse, Error> {
    crate::templates::render_template(
        "paint",
        context! {
        },
    )
}