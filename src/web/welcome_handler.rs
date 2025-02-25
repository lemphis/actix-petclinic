use crate::{model::error_response::ErrorResponse, AppState};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use tera::Context;

#[get("/")]
pub async fn welcome(req: HttpRequest, app_state: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();
    context.insert("current_menu", "home");

    match app_state.tera.render("welcome.html", &context) {
        Ok(html) => HttpResponse::Ok().body(html),
        Err(err) => ErrorResponse::handle_error(&req, &err),
    }
}
