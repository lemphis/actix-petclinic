use actix_web::{http::header::ContentType, web::ServiceConfig, HttpRequest, HttpResponse};
use tera::{Context, Tera};

use crate::model::error_response::ErrorResponse;

pub mod error_handler;
pub mod owner_handler;
pub mod vet_handler;
pub mod welcome_handler;

pub fn configure_route(cfg: &mut ServiceConfig) {
    cfg.service(welcome_handler::welcome)
        .service(vet_handler::show_resources_vet_list)
        .service(vet_handler::show_vet_list)
        .service(owner_handler::show_owner)
        .service(owner_handler::init_creation_form)
        .service(owner_handler::process_creation_form)
        .service(error_handler::trigger_error);
}

pub fn render(
    req: HttpRequest,
    tera: &Tera,
    template_name: &str,
    context: Context,
) -> HttpResponse {
    match tera.render(template_name, &context) {
        Ok(html) => HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(html),
        Err(err) => ErrorResponse::handle_error(&req, Box::new(err)),
    }
}
