use std::{collections::HashMap, sync::LazyLock};

use actix_web::{
    get,
    http::{self},
    post, web, HttpResponse,
};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages, Level};
use regex::Regex;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tera::Context;
use validator::Validate;

use crate::{
    domain::owner::{owners, pet, types},
    model::app_error::AppError,
    web::render,
    AppState,
};

static PHONE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{10}$").unwrap());

#[get("/owners/{id:\\d+}")]
pub async fn show_owner(
    app_state: web::Data<AppState>,
    path: web::Path<u32>,
    messages: IncomingFlashMessages,
) -> Result<HttpResponse, AppError> {
    let owner_id = path.into_inner();
    let conn = &app_state.conn;

    let (owner, pets_with_type) = fetch_owner_with_pets(conn, owner_id).await?;

    let (success_message, error_message) = extract_flash_messages(&messages);

    let mut ctx = Context::new();
    ctx.insert("owner", &owner);
    ctx.insert("pets", &pets_with_type);
    ctx.insert("success_message", &success_message);
    ctx.insert("error_message", &error_message);
    ctx.insert("current_menu", "owners");

    let res = render(&app_state.tera, "owner/owner-details.html", ctx)?;

    Ok(res)
}

fn extract_flash_messages(messages: &IncomingFlashMessages) -> (Option<&str>, Option<&str>) {
    let success_message = messages
        .iter()
        .find(|m| m.level() == Level::Info)
        .map(|m| m.content());
    let error_message = messages
        .iter()
        .find(|m| m.level() == Level::Error)
        .map(|m| m.content());
    (success_message, error_message)
}

async fn fetch_owner_with_pets(
    conn: &sea_orm::DatabaseConnection,
    owner_id: u32,
) -> Result<(owners::Model, Vec<(pet::Model, types::Model)>), AppError> {
    let owner_with_pets = owners::Entity::find_by_id(owner_id)
        .find_with_related(pet::Entity)
        .all(conn)
        .await?;

    let (owner, pets) = owner_with_pets
        .into_iter()
        .next()
        .ok_or(AppError::OwnerNotFound(owner_id))?;

    let pet_type_ids: Vec<u32> = pets.iter().map(|p| p.type_id).collect();
    let pet_types = types::Entity::find()
        .filter(types::Column::Id.is_in(pet_type_ids))
        .all(conn)
        .await?;

    let pets_with_type = join_pets_with_types(pets, pet_types);
    Ok((owner, pets_with_type))
}

fn join_pets_with_types(
    pets: Vec<pet::Model>,
    pet_types: Vec<types::Model>,
) -> Vec<(pet::Model, types::Model)> {
    let type_map: HashMap<u32, types::Model> = pet_types.into_iter().map(|t| (t.id, t)).collect();

    let mut joined_pets: Vec<_> = pets
        .into_iter()
        .filter_map(|pet| type_map.get(&pet.type_id).map(|t| (pet, t.clone())))
        .collect();

    joined_pets.sort_by(|(a, _), (b, _)| a.name.cmp(&b.name));
    joined_pets
}

#[get("/owners/new")]
pub async fn init_creation_form(app_state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let mut ctx = Context::new();
    ctx.insert("current_menu", "owners");

    let res = render(
        &app_state.tera,
        "owner/create-or-update-owner-form.html",
        ctx,
    )?;

    Ok(res)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct CreateNewOwnerRequestForm {
    #[validate(length(min = 1, message = "공백일 수 없습니다"))]
    first_name: String,
    #[validate(length(min = 1, message = "공백일 수 없습니다"))]
    last_name: String,
    #[validate(length(min = 1, message = "공백일 수 없습니다"))]
    address: String,
    #[validate(length(min = 1, message = "공백일 수 없습니다"))]
    city: String,
    #[validate(length(min = 1, message = "공백일 수 없습니다"))]
    #[validate(regex(path = *PHONE_REGEX, message = "Telephone must be a 10-digit number"))]
    telephone: String,
}

#[post("/owners/new")]
pub async fn process_creation_form(
    app_state: web::Data<AppState>,
    form: web::Form<CreateNewOwnerRequestForm>,
) -> Result<HttpResponse, AppError> {
    let owner = form.into_inner();

    if let Err(errors) = owner.validate() {
        let mut errors_map: HashMap<String, Vec<String>> = HashMap::new();
        for (field, errors) in errors.field_errors().iter() {
            let msgs = errors
                .iter()
                .map(|e| {
                    e.message
                        .clone()
                        .unwrap_or_else(|| "잘못된 입력".into())
                        .to_string()
                })
                .collect();
            errors_map.insert(field.to_string(), msgs);
        }

        let mut ctx = Context::new();
        ctx.insert("current_menu", "owners");
        ctx.insert("owner", &owner);
        ctx.insert("errors", &errors_map);

        return render(
            &app_state.tera,
            "owner/create-or-update-owner-form.html",
            ctx,
        );
    }

    let new_owner = owners::ActiveModel {
        first_name: ActiveValue::Set(Some(owner.first_name)),
        last_name: ActiveValue::Set(Some(owner.last_name)),
        address: ActiveValue::Set(Some(owner.address)),
        city: ActiveValue::Set(Some(owner.city)),
        telephone: ActiveValue::Set(Some(owner.telephone)),
        ..Default::default()
    };
    let insert_result = owners::Entity::insert(new_owner.clone())
        .exec(&app_state.conn)
        .await?;

    FlashMessage::info("New Owner Created").send();
    let redirect_url = format!("/owners/{}", insert_result.last_insert_id);
    let res = HttpResponse::Found()
        .append_header((http::header::LOCATION, redirect_url))
        .finish();

    Ok(res)
}
