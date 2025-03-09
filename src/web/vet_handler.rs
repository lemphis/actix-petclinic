use crate::{
    domain::veterinarian::{specialty, vet, vet_specialty},
    model::{app_error::AppError, page::Page},
    web::render,
    AppState,
};
use actix_web::{get, web, HttpResponse};
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
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let vet_list = fetch_all_vet_specialties(&app_state.conn).await?;

    let response = ShowResourcesVetListResponse { vet_list };
    let xml = quick_xml::se::to_string(&response)?;
    let res = HttpResponse::Ok().content_type("application/xml").body(xml);

    Ok(res)
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
    app_state: web::Data<AppState>,
    query: web::Query<ShowVetListQuery>,
) -> Result<HttpResponse, AppError> {
    let conn = &app_state.conn;
    let (cur_page, size) = (query.page.unwrap_or(1), query.size.unwrap_or(5));
    let (vet_list, vet_total_count) = tokio::try_join!(
        fetch_vet_specialties_with_pagination(conn, cur_page, size),
        fetch_vet_total_count(conn)
    )?;

    let page = Page::new(cur_page, vet_total_count);
    let mut ctx = Context::new();
    ctx.insert("vets", &vet_list);
    ctx.insert("page", &cur_page);
    ctx.insert("total_pages", &page.total_pages());
    ctx.insert("has_previous", &page.has_previous());
    ctx.insert("has_next", &page.has_next());
    ctx.insert("page_range", page.page_range());
    ctx.insert("current_menu", "vets");

    let res = render(&app_state.tera, "vet/vet-list.html", ctx)?;

    Ok(res)
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
