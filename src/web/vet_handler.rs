use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{
    model::{app_error::AppError, page::Page},
    service::vet_service::{self, VetService, VetWithSpecialties},
    web::render,
    AppState,
};

#[derive(Serialize)]
#[serde(rename = "vets")]
#[serde(rename_all = "camelCase")]
struct ShowResourcesVetListResponse {
    vet_list: Vec<Vet>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Vet {
    id: u32,
    first_name: Option<String>,
    last_name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    specialties: Vec<self::Specialty>,
}

impl From<VetWithSpecialties> for Vet {
    fn from(value: VetWithSpecialties) -> Self {
        Vet {
            id: value.vet_id,
            first_name: value.first_name,
            last_name: value.last_name,
            specialties: value.specialties.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct Specialty {
    id: u32,
    name: Option<String>,
}

impl From<vet_service::Specialty> for self::Specialty {
    fn from(value: vet_service::Specialty) -> Self {
        self::Specialty {
            id: value.specialty_id,
            name: value.specialty_name,
        }
    }
}

#[get("/vets")]
pub async fn show_resources_vet_list(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let AppState { conn, .. } = app_state.get_ref();

    let vet_list = VetService::fetch_all_vets_with_specialties(conn).await?;

    let response = ShowResourcesVetListResponse {
        vet_list: vet_list.into_iter().map(Into::into).collect(),
    };
    let xml = quick_xml::se::to_string(&response)?;
    let res = HttpResponse::Ok().content_type("application/xml").body(xml);

    Ok(res)
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
    let AppState { conn, tera, .. } = app_state.get_ref();

    let (cur_page, size) = (query.page.unwrap_or(1), query.size.unwrap_or(5));

    let vet_total_count = VetService::fetch_all_vets_count(conn).await?;
    let vet_list = if vet_total_count > 0 {
        VetService::fetch_vets_with_specialties_paginated(conn, cur_page, size).await?
    } else {
        vec![]
    };

    let page = Page::new(cur_page, vet_total_count);
    let mut ctx = Context::new();
    ctx.insert("vets", &vet_list);
    ctx.insert("page", &cur_page);
    ctx.insert("total_pages", &page.total_pages());
    ctx.insert("has_previous", &page.has_previous());
    ctx.insert("has_next", &page.has_next());
    ctx.insert("page_range", page.page_range());
    ctx.insert("query_params", &Vec::<(&str, String)>::new());
    ctx.insert("current_menu", "vets");

    render(tera, "vet/vet-list.html", ctx)
}
