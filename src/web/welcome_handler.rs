use crate::{model::app_error::AppError, web::render, AppState};
use actix_web::{get, web, HttpRequest, HttpResponse};
use tera::Context;

#[get("/")]
pub async fn welcome(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let translation = app_state.i18n.get(&req);

    let mut context = Context::new();
    context.insert("current_menu", "home");
    context.insert("translation", translation);

    let res = render(&app_state.tera, "welcome.html", context)?;

    Ok(res)
}
