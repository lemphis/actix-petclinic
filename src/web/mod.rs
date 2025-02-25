use actix_web::web::ServiceConfig;

pub mod error_handler;
pub mod vet_handler;
pub mod welcome_handler;

pub fn configure_route(cfg: &mut ServiceConfig) {
    cfg.service(welcome_handler::welcome)
        .service(vet_handler::show_resources_vet_list)
        .service(vet_handler::show_vet_list)
        .service(error_handler::trigger_error);
}
