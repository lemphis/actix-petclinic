use crate::{
    domain::owner::owners,
    model::app_error::AppError,
    service::{owner_service::OwnerService, pet_service::PetService},
    web::render,
    AppState,
};
use actix_web::{get, web, HttpResponse};
use sea_orm::DbConn;
use tera::Context;
use tokio::try_join;

#[get(r"/owners/{owner_id:\d+}/pets/new")]
pub async fn init_creation_form(
    app_state: web::Data<AppState>,
    path: web::Path<u32>,
) -> Result<HttpResponse, AppError> {
    let AppState { conn, tera, .. } = app_state.get_ref();

    let owner_id = path.into_inner();

    let (owner, pet_type_names) = get_owner_and_pet_types(conn, owner_id).await?;

    let mut ctx = Context::new();
    ctx.insert("current_menu", "owners");
    ctx.insert("owner", &owner);
    ctx.insert("pet_types", &pet_type_names);

    render(tera, "pet/create-or-update-pet-form.html", ctx)
}

async fn get_owner_and_pet_types(
    conn: &DbConn,
    owner_id: u32,
) -> Result<(owners::Model, Vec<String>), AppError> {
    let (owner, pet_types) = try_join!(
        OwnerService::fetch_owner_by_id(conn, owner_id),
        PetService::fetch_all_pet_types(conn)
    )?;

    let pet_type_names: Vec<String> = pet_types.into_iter().filter_map(|t| t.name).collect();

    Ok((owner, pet_type_names))
}
