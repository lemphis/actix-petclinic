use actix_web::{get, post, web, HttpRequest, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tera::Context;
use validator::Validate;

use crate::{
    model::app_error::AppError,
    service::{owner_service::OwnerService, visit_service::VisitService},
    web::{redirect, render, validator::validate_future_date},
    AppState,
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

#[derive(Serialize, Deserialize, Validate)]
struct CreateVisitForm {
    #[validate(custom(function = validate_future_date))]
    date: String,
    #[validate(length(min = 1, message = "공백일 수 없습니다"))]
    description: String,
}

#[post(r"/owners/{owner_id:\d+}/pets/{pet_id:\d+}/visits/new")]
pub async fn process_new_visit_form(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    path: web::Path<OwnerWithPetPathParams>,
    form: web::Form<CreateVisitForm>,
) -> Result<HttpResponse, AppError> {
    let AppState { conn, tera, i18n } = app_state.get_ref();

    let OwnerWithPetPathParams { owner_id, pet_id } = path.into_inner();
    let create_visit_form = form.into_inner();

    if let Err(errors) = create_visit_form.validate() {
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

        let translated_errors = i18n.translate_errors(&req, &errors);

        let mut ctx = Context::new();
        ctx.insert("errors", &translated_errors);
        ctx.insert("owner", &owner_with_pets_and_types);
        ctx.insert("pet", pet);
        ctx.insert("visit", &create_visit_form);
        ctx.insert("current_menu", "owners");

        return render(tera, "pet/create-or-update-visit-form.html", ctx);
    }

    // form data 검증 시 확인하였으므로 반드시 Some임
    let visit_date = NaiveDate::parse_from_str(&create_visit_form.date, "%Y-%m-%d").unwrap();

    VisitService::save_visit(
        conn,
        Some(pet_id),
        Some(visit_date),
        Some(create_visit_form.description),
    )
    .await?;

    FlashMessage::info("Your visit has been booked").send();

    Ok(redirect(format!("/owners/{owner_id}")))
}
