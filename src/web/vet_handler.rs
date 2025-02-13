use crate::{
    domain::veterinarian::{specialty, vet, vet_specialty},
    model::error_response::ErrorResponse,
    AppState,
};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use quick_xml::se::to_string;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

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
    let conn = &app_state.db;

    let vet_list = fetch_vet_data(conn).await;
    if let Err(db_err) = vet_list {
        return ErrorResponse::handle_db_error(&req, &db_err);
    }

    let response = ShowResourcesVetListResponse {
        vet_list: vet_list.unwrap(),
    };

    HttpResponse::Ok()
        .content_type("application/xml")
        .body(to_string(&response).unwrap())
}

async fn fetch_vet_data(conn: &DatabaseConnection) -> Result<Vec<Vet>, sea_orm::DbErr> {
    let vets = fetch_vets(conn).await?;

    let vet_ids = vets.clone().iter().map(|vet| vet.id).collect::<Vec<_>>();
    let vet_specialties = fetch_vet_specialties(conn, &vet_ids).await?;

    let specialty_ids = vet_specialties
        .iter()
        .map(|vs| vs.specialty_id)
        .collect::<Vec<_>>();
    let specialties = fetch_specialties(conn, &specialty_ids).await?;

    let vet_list = vets
        .iter()
        .map(|vet| create_vet_response(vet.clone(), &vet_specialties, &specialties))
        .collect();

    Ok(vet_list)
}

async fn fetch_vets(conn: &DatabaseConnection) -> Result<Vec<vet::Model>, sea_orm::DbErr> {
    vet::Entity::find().all(conn).await
}

async fn fetch_vet_specialties(
    conn: &DatabaseConnection,
    vet_ids: &[u32],
) -> Result<Vec<vet_specialty::Model>, sea_orm::DbErr> {
    vet_specialty::Entity::find()
        .filter(vet_specialty::Column::VetId.is_in(vet_ids.to_owned()))
        .all(conn)
        .await
}

async fn fetch_specialties(
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
    let specialties: Vec<specialty::Model> = vet_specialties
        .iter()
        .filter(|vs| vet.id == vs.vet_id)
        .filter_map(|vs| find_specialty(specialties, vs.specialty_id))
        .collect();

    Vet {
        id: vet.id,
        first_name: vet.first_name,
        last_name: vet.last_name,
        specialties,
    }
}

fn find_specialty(specialties: &[specialty::Model], specialty_id: u32) -> Option<specialty::Model> {
    specialties.iter().find(|s| specialty_id == s.id).cloned()
}
