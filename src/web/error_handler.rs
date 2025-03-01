use actix_web::{get, http::header::ContentType, web, HttpRequest, HttpResponse, Responder};
use tera::Context;

use crate::{model::error_response::ErrorResponse, AppState};

#[get("/oups")]
pub async fn trigger_error(req: HttpRequest, app_state: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();
    context.insert("current_menu", "error");
    context.insert(
        "message",
        "Expected: handler used to showcase what happens when an error is propagated",
    );

    match app_state.tera.render("error.html", &context) {
        Ok(html) => HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(html),
        Err(err) => ErrorResponse::handle_error(&req, &err),
    }
}
