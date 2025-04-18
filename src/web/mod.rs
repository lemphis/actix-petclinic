use actix_web::{
    http::{self, header::ContentType},
    web::ServiceConfig,
    HttpResponse,
};
use tera::{Context, Tera};

use crate::model::app_error::AppError;

pub mod error_handler;
pub mod owner_handler;
pub mod pet_handler;
pub mod validator;
pub mod vet_handler;
pub mod visit_handler;
pub mod welcome_handler;

pub fn configure_route(cfg: &mut ServiceConfig) {
    cfg.service(welcome_handler::welcome)
        .service(vet_handler::show_resources_vet_list)
        .service(vet_handler::show_vet_list)
        .service(owner_handler::show_owner)
        .service(owner_handler::init_creation_form)
        .service(owner_handler::process_creation_form)
        .service(owner_handler::init_find_form)
        .service(owner_handler::process_find_form)
        .service(owner_handler::init_update_owner_form)
        .service(owner_handler::process_update_owner_form)
        .service(pet_handler::init_creation_form)
        .service(pet_handler::process_creation_form)
        .service(pet_handler::init_update_form)
        .service(pet_handler::process_update_form)
        .service(visit_handler::init_new_visit_form)
        .service(visit_handler::process_new_visit_form)
        .service(error_handler::trigger_error);
}

pub fn render(
    tera: &Tera,
    template_name: &str,
    context: Context,
) -> Result<HttpResponse, AppError> {
    let html = tera.render(template_name, &context)?;
    let res = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html);

    Ok(res)
}

pub fn redirect(path: String) -> HttpResponse {
    HttpResponse::Found()
        .append_header((http::header::LOCATION, path))
        .finish()
}
