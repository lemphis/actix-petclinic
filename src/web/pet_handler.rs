use crate::{
    domain::owner::owners,
    model::app_error::AppError,
    service::{
        owner_service::{OwnerService, PetWithTypeAndVisits},
        pet_service::PetService,
    },
    web::{redirect, render, validator::create_validation_error},
    AppState,
};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use chrono::NaiveDate;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use tera::Context;
use tokio::try_join;
use validator::{Validate, ValidationErrors};

use super::validator::{validate_not_blank, validate_pet_type, validate_today_or_past_date};

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
    ctx.insert("is_new", &true);

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

#[derive(Serialize, Deserialize, Validate)]
struct CreateOrUpdatePetForm {
    #[validate(custom(function = validate_not_blank))]
    pet_name: String,
    #[validate(custom(function = validate_today_or_past_date))]
    birth_date: String,
    #[validate(custom(function = validate_pet_type))]
    pet_type: String,
}

#[post(r"/owners/{owner_id:\d+}/pets/new")]
pub async fn process_creation_form(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    path: web::Path<u32>,
    form: web::Form<CreateOrUpdatePetForm>,
) -> Result<HttpResponse, AppError> {
    let AppState { conn, .. } = app_state.get_ref();

    let owner_id = path.into_inner();
    let create_pet_form = form.into_inner();

    let errors = {
        let mut all_errors = ValidationErrors::new();

        if let Err(errors) = create_pet_form.validate() {
            all_errors = errors;
        }

        let owner_with_pets =
            OwnerService::fetch_owner_with_pets_and_types_and_visits_by_owner_id(conn, owner_id)
                .await?;
        if owner_with_pets
            .pets_with_type
            .into_iter()
            .filter_map(|p| p.pet_name)
            .any(|name| name.to_lowercase() == create_pet_form.pet_name.to_lowercase())
        {
            all_errors.add(
                "pet_name",
                create_validation_error("duplicate", "duplicate"),
            );
        }

        all_errors
    };

    if !errors.is_empty() {
        return render_pet_form_with_errors(
            &req,
            app_state,
            owner_id,
            create_pet_form,
            errors,
            true,
        )
        .await;
    }

    let pet_type_id = PetService::fetch_all_pet_types(conn)
        .await?
        .iter()
        .find(|t| t.name == Some(create_pet_form.pet_type.clone()))
        .map(|t| t.id)
        .unwrap(); // pet마다 pet type이 반드시 존재하므로 Some임

    // form data 검증 시 확인하였으므로 반드시 Some임
    let birth_date = NaiveDate::parse_from_str(&create_pet_form.birth_date, "%Y-%m-%d").unwrap();

    PetService::save_pet(
        conn,
        Some(create_pet_form.pet_name),
        Some(birth_date),
        pet_type_id,
        Some(owner_id),
    )
    .await?;

    FlashMessage::info("New Pet has been Added").send();

    Ok(redirect(format!("/owners/{owner_id}")))
}

async fn render_pet_form_with_errors(
    req: &HttpRequest,
    app_state: web::Data<AppState>,
    owner_id: u32,
    pet_form: CreateOrUpdatePetForm,
    errors: validator::ValidationErrors,
    is_new: bool,
) -> Result<HttpResponse, AppError> {
    let AppState { conn, tera, i18n } = app_state.get_ref();

    let translated_errors = i18n.translate_errors(req, &errors);

    let (owner, pet_type_names) = get_owner_and_pet_types(conn, owner_id).await?;

    let mut ctx = Context::new();
    ctx.insert("current_menu", "owners");
    ctx.insert("owner", &owner);
    ctx.insert("pet", &pet_form);
    ctx.insert("pet_types", &pet_type_names);
    ctx.insert("errors", &translated_errors);
    ctx.insert("is_new", &is_new);

    render(tera, "pet/create-or-update-pet-form.html", ctx)
}

#[derive(Deserialize)]
struct OwnerWithPetPathParams {
    owner_id: u32,
    pet_id: u32,
}

#[get(r"/owners/{owner_id:\d+}/pets/{pet_id:\d+}/edit")]
pub async fn init_update_form(
    app_state: web::Data<AppState>,
    path: web::Path<OwnerWithPetPathParams>,
) -> Result<HttpResponse, AppError> {
    let AppState { conn, tera, .. } = app_state.get_ref();

    let OwnerWithPetPathParams { owner_id, pet_id } = path.into_inner();

    let (owner_with_pets, pet_types) = try_join!(
        OwnerService::fetch_owner_with_pets_and_types_and_visits_by_owner_id(conn, owner_id),
        PetService::fetch_all_pet_types(conn)
    )?;

    let PetWithTypeAndVisits {
        pet_name,
        birth_date,
        pet_type,
        ..
    } = find_pet_by_id(&owner_with_pets.pets_with_type, pet_id)?;
    // pet name, birth_date, pet type name은 form data를 검증하기 때문에 반드시 존재하므로 Some임
    let pet_form = CreateOrUpdatePetForm {
        pet_name: pet_name.clone().unwrap(),
        birth_date: birth_date.unwrap().to_string(),
        pet_type: pet_type.type_name.clone().unwrap(),
    };
    let pet_type_names: Vec<String> = pet_types.into_iter().filter_map(|t| t.name).collect();

    let mut ctx = Context::new();
    ctx.insert("current_menu", "owners");
    ctx.insert("owner", &owner_with_pets);
    ctx.insert("pet", &pet_form);
    ctx.insert("pet_types", &pet_type_names);

    render(tera, "pet/create-or-update-pet-form.html", ctx)
}

fn find_pet_by_id(
    pets: &[PetWithTypeAndVisits],
    pet_id: u32,
) -> Result<&PetWithTypeAndVisits, AppError> {
    pets.iter()
        .find(|p| p.pet_id == pet_id)
        .ok_or_else(|| AppError::ResourceNotFound {
            resource: "pet".to_string(),
            id: pet_id,
        })
}
