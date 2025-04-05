use crate::{model::app_error::AppError, web::render, AppState};
use actix_web::{get, web, HttpRequest, HttpResponse};
use tera::Context;

#[get("/")]
pub async fn welcome(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let welcome_msg = app_state.i18n.translate(&req, "welcome");

    let mut ctx = Context::new();
    ctx.insert("current_menu", "home");
    ctx.insert("welcome_msg", &welcome_msg);

    render(&app_state.tera, "welcome.html", ctx)
}
