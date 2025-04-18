use actix_web::{get, web, HttpResponse};
use tera::Context;

use crate::{model::app_error::AppError, web::render, AppState};

#[get("/oups")]
pub async fn trigger_error(app_state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let AppState { tera, .. } = app_state.get_ref();

    let mut ctx = Context::new();
    ctx.insert("current_menu", "error");
    ctx.insert(
        "message",
        "Expected: handler used to showcase what happens when an error is propagated",
    );

    render(tera, "error.html", ctx)
}
