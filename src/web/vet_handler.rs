use crate::{
    domain::veterinarian::{specialty, vet, vet_specialty},
    model::{error_response::ErrorResponse, page::Page},
    AppState,
};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use quick_xml::se::to_string;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tera::Context;

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

#[derive(Deserialize, Clone)]
struct ShowVetListQuery {
    page: Option<u64>,
    size: Option<u64>,
}

#[get("/vets.html")]
pub async fn show_vet_list(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    query: web::Query<ShowVetListQuery>,
) -> impl Responder {
    let conn = &app_state.conn;
    let (cur_page, size) = (query.page.unwrap_or(1), query.size.unwrap_or(5));
    let (vet_list, vet_total_count) = match tokio::try_join!(
        fetch_vet_specialties_with_pagination(conn, cur_page, size),
        fetch_vet_total_count(conn)
    ) {
        Ok((vets, count)) => (vets, count),
        Err(db_err) => return ErrorResponse::handle_error(&req, &db_err),
    };

    let page = Page::new(cur_page, vet_total_count);
    let mut context = Context::new();
    context.insert("vets", &vet_list);
    context.insert("page", &cur_page);
    context.insert("total_pages", &page.total_pages());
    context.insert("has_previous", &page.has_previous());
    context.insert("has_next", &page.has_next());
    context.insert("page_range", page.page_range());

    match app_state.tera.render("vet/vet-list.html", &context) {
        Ok(html) => HttpResponse::Ok().body(html),
        Err(err) => ErrorResponse::handle_error(&req, &err),
    }
}

async fn fetch_vet_specialties_with_pagination(
    conn: &DatabaseConnection,
    page: u64,
    size: u64,
) -> Result<Vec<Vet>, sea_orm::DbErr> {
    let vets = fetch_vets_with_pagination(conn, page, size).await?;
    fetch_specialties_by_vets(conn, &vets).await
}

async fn fetch_vets_with_pagination(
    conn: &DatabaseConnection,
    page: u64,
    size: u64,
) -> Result<Vec<vet::Model>, sea_orm::DbErr> {
    vet::Entity::find()
        .order_by_asc(vet::Column::Id)
        .paginate(conn, size)
        .fetch_page(page - 1)
        .await
}

async fn fetch_vet_total_count(conn: &DatabaseConnection) -> Result<u64, sea_orm::DbErr> {
    vet::Entity::find().count(conn).await
}
