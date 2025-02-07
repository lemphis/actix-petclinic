use crate::{
    domain::veterinarian::{specialty, vet, vet_specialty},
    model::error_response::ErrorResponse,
};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct ShowResourcesVetListQuery {
    page: u64,
    size: Option<u64>,
}

#[derive(Serialize, Clone)]
struct ShowResourcesVetListResponse {
    vets: VetList,
}

#[derive(Serialize, Clone)]
struct VetList {
    vet_list: Vec<Vet>,
}

#[derive(Serialize, Clone)]
struct Vet {
    id: u32,
    first_name: Option<String>,
    last_name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    specialties: Vec<specialty::Model>,
}

#[get("/vets")]
pub async fn show_resources_vet_list(
    connection: web::Data<Arc<DatabaseConnection>>,
    req: HttpRequest,
    query: web::Query<ShowResourcesVetListQuery>,
) -> impl Responder {
    let conn = connection.as_ref().as_ref();

    let vet_list = fetch_vet_data(conn, query.page, query.size.unwrap_or(5)).await;
    if let Err(db_err) = vet_list {
        return handle_db_error(&req, db_err);
    }

    let response = ShowResourcesVetListResponse {
        vets: VetList {
            vet_list: vet_list.unwrap(),
        },
    };

    HttpResponse::Ok().json(response)
}

async fn fetch_vet_data(
    conn: &DatabaseConnection,
    page: u64,
    size: u64,
) -> Result<Vec<Vet>, sea_orm::DbErr> {
    let vets = fetch_vets(conn, page, size).await?;

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

async fn fetch_vets(
    conn: &DatabaseConnection,
    page: u64,
    size: u64,
) -> Result<Vec<vet::Model>, sea_orm::DbErr> {
    vet::Entity::find()
        .paginate(conn, size)
        .fetch_page(page - 1)
        .await
}

fn handle_db_error(req: &HttpRequest, db_err: sea_orm::DbErr) -> HttpResponse {
    HttpResponse::InternalServerError().json(ErrorResponse::new(
        db_err.to_string(),
        req.uri().to_string(),
    ))
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
