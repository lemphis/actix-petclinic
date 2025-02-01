use crate::{domain::veterinarian::vet, model::error_response::ErrorResponse};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
struct ShowResourcesVetListResponse {
    vets: Vec<vet::Model>,
}

#[get("/vets")]
pub async fn show_resources_vet_list(
    connection: web::Data<Arc<DatabaseConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let vets = vet::Entity::find().all(connection.as_ref().as_ref()).await;

    match vets {
        Ok(vets) => HttpResponse::Ok().json(ShowResourcesVetListResponse { vets }),
        Err(db_err) => HttpResponse::InternalServerError().json(ErrorResponse::new(
            db_err.to_string(),
            req.uri().to_string(),
        )),
    }
}
