use crate::{
    domain::veterinarian::{specialty, vet, vet_specialty},
    model::error_response::ErrorResponse,
    AppState,
};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use quick_xml::se::to_string;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
#[serde(rename = "vets")]
#[serde(rename_all = "camelCase")]
struct ShowResourcesVetListResponse {
    vet_list: Vec<Vet>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Vet {
    id: u32,
    first_name: Option<String>,
    last_name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    specialties: Vec<specialty::Model>,
}

#[get("/vets")]
pub async fn show_resources_vet_list(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let vet_list = match fetch_all_vet_specialties(&app_state.conn).await {
        Ok(vets) => vets,
        Err(db_err) => return ErrorResponse::handle_error(&req, &db_err),
    };

    let response = ShowResourcesVetListResponse { vet_list };
    match to_string(&response) {
        Ok(xml) => HttpResponse::Ok().content_type("application/xml").body(xml),
        Err(err) => ErrorResponse::handle_error(&req, &err),
    }
}

async fn fetch_all_vet_specialties(conn: &DatabaseConnection) -> Result<Vec<Vet>, sea_orm::DbErr> {
    let vets = fetch_all_vets(conn).await?;
    fetch_specialties_by_vets(conn, &vets).await
}

async fn fetch_all_vets(conn: &DatabaseConnection) -> Result<Vec<vet::Model>, sea_orm::DbErr> {
    vet::Entity::find().all(conn).await
}

async fn fetch_specialties_by_vets(
    conn: &DatabaseConnection,
    vets: &[vet::Model],
) -> Result<Vec<Vet>, sea_orm::DbErr> {
    let vet_ids = vets.iter().map(|vet| vet.id).collect::<Vec<_>>();
    let vet_specialties = fetch_vet_specialties_by_vet_ids(conn, &vet_ids).await?;

    let specialty_ids = vet_specialties
        .iter()
        .map(|vs| vs.specialty_id)
        .collect::<Vec<_>>();
    let specialties = fetch_specialties_by_specialty_ids(conn, &specialty_ids).await?;

    let vet_list = vets
        .iter()
        .map(|vet| create_vet_response(vet.clone(), &vet_specialties, &specialties))
        .collect();

    Ok(vet_list)
}

async fn fetch_vet_specialties_by_vet_ids(
    conn: &DatabaseConnection,
    vet_ids: &[u32],
) -> Result<Vec<vet_specialty::Model>, sea_orm::DbErr> {
    vet_specialty::Entity::find()
        .filter(vet_specialty::Column::VetId.is_in(vet_ids.to_owned()))
        .all(conn)
        .await
}

async fn fetch_specialties_by_specialty_ids(
    conn: &DatabaseConnection,
    specialty_ids: &[u32],
) -> Result<Vec<specialty::Model>, sea_orm::DbErr> {
    specialty::Entity::find()
        .filter(specialty::Column::Id.is_in(specialty_ids.to_owned()))
        .all(conn)
        .await
}

fn create_vet_response(
    vet: vet::Model,
    vet_specialties: &[vet_specialty::Model],
    specialties: &[specialty::Model],
) -> Vet {
    let specialty_map: HashMap<u32, specialty::Model> =
        specialties.iter().cloned().map(|s| (s.id, s)).collect();

    let specialties: Vec<specialty::Model> = vet_specialties
        .iter()
        .filter(|vs| vet.id == vs.vet_id)
        .filter_map(|vs| specialty_map.get(&vs.specialty_id).cloned())
        .collect();

    Vet {
        id: vet.id,
        first_name: vet.first_name,
        last_name: vet.last_name,
        specialties,
    }
}
