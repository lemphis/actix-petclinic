use crate::{
    model::app_error::AppError,
    service::{owner_service::OwnerService, pet_service::PetService},
    web::render,
    AppState,
};
use actix_web::{get, web, HttpResponse};
use tera::Context;

#[get(r"/owners/{owner_id:\d+}/pets/new")]
pub async fn init_creation_form(
    app_state: web::Data<AppState>,
    path: web::Path<u32>,
) -> Result<HttpResponse, AppError> {
    let owner_id = path.into_inner();
    let owner = OwnerService::fetch_owner_by_id(&app_state.conn, owner_id).await?;
    let pet_type_names: Vec<String> = PetService::fetch_all_pet_types(&app_state.conn)
        .await?
        .into_iter()
        .filter_map(|t| t.name)
        .collect();

    let mut ctx = Context::new();
    ctx.insert("current_menu", "owners");
    ctx.insert("owner", &owner);
    ctx.insert("pet_types", &pet_type_names);

    render(&app_state.tera, "pet/create-or-update-pet-form.html", ctx)
}
