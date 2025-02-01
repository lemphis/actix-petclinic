use actix_web::web::ServiceConfig;
use vet_handler::show_resources_vet_list;

pub mod vet_handler;

pub fn configure_route(cfg: &mut ServiceConfig) {
    cfg.service(show_resources_vet_list);
}
