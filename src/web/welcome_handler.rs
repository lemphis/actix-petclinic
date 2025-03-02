use crate::{web::render, AppState};
use actix_web::{get, web, HttpRequest, Responder};
use tera::Context;

#[get("/")]
pub async fn welcome(req: HttpRequest, app_state: web::Data<AppState>) -> impl Responder {
    let translation = app_state.i18n.get(&req);

    let mut context = Context::new();
    context.insert("current_menu", "home");
    context.insert("translation", translation);

    render(req, &app_state.tera, "welcome.html", context)
}
