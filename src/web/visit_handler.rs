use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use tera::Context;

use crate::{
    model::app_error::AppError, service::owner_service::OwnerService, web::render, AppState,
};

#[derive(Deserialize)]
struct OwnerWithPetPathParams {
    owner_id: u32,
    pet_id: u32,
}

#[get(r"/owners/{owner_id:\d+}/pets/{pet_id:\d+}/visits/new")]
pub async fn init_new_visit_form(
    app_state: web::Data<AppState>,
    path: web::Path<OwnerWithPetPathParams>,
) -> Result<HttpResponse, AppError> {
    let AppState { conn, tera, .. } = app_state.get_ref();

    let OwnerWithPetPathParams { owner_id, pet_id } = path.into_inner();

    let owner_with_pets_and_types =
        OwnerService::fetch_owner_with_pets_and_types_and_visits_by_owner_id(conn, owner_id)
            .await?;

    let pet = owner_with_pets_and_types
        .pets_with_type
        .iter()
        .find(|p| p.pet_id == pet_id)
        .ok_or_else(|| AppError::ResourceNotFound {
            resource: "pet".to_string(),
            id: pet_id,
        })?;

    let mut ctx = Context::new();
    ctx.insert("owner", &owner_with_pets_and_types);
    ctx.insert("pet", pet);
    ctx.insert("current_menu", "owners");

    render(tera, "pet/create-or-update-visit-form.html", ctx)
}
