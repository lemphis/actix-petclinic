use crate::{model::error_response::ErrorResponse, AppState};
use actix_web::{get, http::header::ContentType, web, HttpRequest, HttpResponse, Responder};
use tera::Context;

#[get("/")]
pub async fn welcome(req: HttpRequest, app_state: web::Data<AppState>) -> impl Responder {
    let translation = app_state.i18n.get(&req);

    let mut context = Context::new();
    context.insert("current_menu", "home");
    context.insert("translation", translation);

    match app_state.tera.render("welcome.html", &context) {
        Ok(html) => HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(html),
        Err(err) => ErrorResponse::handle_error(&req, &err),
    }
}
