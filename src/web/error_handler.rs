use actix_web::{get, web, HttpRequest, Responder};
use tera::Context;

use crate::{web::render, AppState};

#[get("/oups")]
pub async fn trigger_error(req: HttpRequest, app_state: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();
    context.insert("current_menu", "error");
    context.insert(
        "message",
        "Expected: handler used to showcase what happens when an error is propagated",
    );

    render(req, &app_state.tera, "error.html", context)
}
