use std::{collections::HashMap, sync::LazyLock};

use actix_web::{
    get,
    http::{self},
    post, web, HttpRequest, HttpResponse,
};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages, Level};
use regex::Regex;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};
use tera::Context;
use validator::Validate;

use crate::{
    domain::owner::{self, owners, pet, types},
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

    render(&app_state.tera, "owner/owner-details.html", ctx)
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
        .ok_or(AppError::ResourceNotFound {
            resource: "owner".to_string(),
            id: owner_id,
        })?;

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

    let mut joined_pets: Vec<(pet::Model, owner::types::Model)> = pets
        .into_iter()
        .filter_map(|pet| type_map.get(&pet.type_id).map(|t| (pet, t.clone())))
        .collect();

    joined_pets.sort_by(|(a, _), (b, _)| a.name.cmp(&b.name));

    joined_pets
}

fn extract_flash_messages(messages: &IncomingFlashMessages) -> (Option<&str>, Option<&str>) {
    let (mut success_message, mut error_message) = (None, None);

    for message in messages.iter() {
        match message.level() {
            Level::Info => success_message = Some(message.content()),
            Level::Error => error_message = Some(message.content()),
            _ => {}
        }
    }

    (success_message, error_message)
}

#[get("/owners/new")]
pub async fn init_creation_form(app_state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let mut ctx = Context::new();
    ctx.insert("current_menu", "owners");

    render(
        &app_state.tera,
        "owner/create-or-update-owner-form.html",
        ctx,
    )
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct CreateOwnerForm {
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
    form: web::Form<CreateOwnerForm>,
) -> Result<HttpResponse, AppError> {
    let owner = form.into_inner();

    if let Err(errors) = owner.validate() {
        return handle_validation_errors(&app_state.tera, owner, errors);
    }

    let last_insert_id = create_owner(&app_state.conn, owner).await?;

    FlashMessage::info("New Owner Created").send();
    let res = HttpResponse::Found()
        .append_header((http::header::LOCATION, format!("/owners/{last_insert_id}")))
        .finish();

    Ok(res)
}

fn handle_validation_errors(
    tera: &tera::Tera,
    owner: CreateOwnerForm,
    errors: validator::ValidationErrors,
) -> Result<HttpResponse, AppError> {
    let mut errors_map: HashMap<String, Vec<String>> = HashMap::new();

    for (field, field_errors) in errors.field_errors().iter() {
        let error_messages = field_errors
            .iter()
            .map(|e| {
                e.message
                    .clone()
                    .unwrap_or_else(|| "잘못된 입력".into())
                    .to_string()
            })
            .collect();

        errors_map.insert(field.to_string(), error_messages);
    }

    let mut ctx = Context::new();
    ctx.insert("current_menu", "owners");
    ctx.insert("owner", &owner);
    ctx.insert("errors", &errors_map);

    render(tera, "owner/create-or-update-owner-form.html", ctx)
}

async fn create_owner(
    conn: &sea_orm::DatabaseConnection,
    owner: CreateOwnerForm,
) -> Result<u32, AppError> {
    let new_owner = owners::ActiveModel {
        first_name: ActiveValue::Set(Some(owner.first_name)),
        last_name: ActiveValue::Set(Some(owner.last_name)),
        address: ActiveValue::Set(Some(owner.address)),
        city: ActiveValue::Set(Some(owner.city)),
        telephone: ActiveValue::Set(Some(owner.telephone)),
        ..Default::default()
    };

    let result = owners::Entity::insert(new_owner).exec(conn).await?;

    Ok(result.last_insert_id)
}

#[get("/owners/find")]
pub async fn init_find_form(app_state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let mut ctx = Context::new();
    ctx.insert("current_menu", "owners");

    render(&app_state.tera, "owner/find-owners.html", ctx)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct FindOwnerRequestQuery {
    last_name: Option<String>,
    page: Option<u64>,
    size: Option<u64>,
}

#[get("/owners")]
pub async fn process_find_form(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    query: web::Query<FindOwnerRequestQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query.into_inner();
    let (last_name, cur_page, size) = (
        query.last_name.unwrap_or("".to_string()),
        query.page.unwrap_or(1),
        query.size.unwrap_or(5),
    );
    let conn = &app_state.conn;

    let mut ctx = Context::new();
    ctx.insert("current_menu", "owners");

    let owner_with_pets = owners::Entity::find()
        .left_join(pet::Entity)
        .filter(owners::Column::LastName.like(format!("{last_name}%")))
        .limit(size)
        .select_also(pet::Entity)
        .all(conn)
        .await?;

    if owner_with_pets.is_empty() {
        let translation = app_state.i18n.get(&req);
        ctx.insert("translation", translation);
        ctx.insert("last_name", &last_name);

        return render(&app_state.tera, "owner/find-owners.html", ctx);
    }

    if owner_with_pets.len() == 1 {
        let res = HttpResponse::Found()
            .append_header((
                http::header::LOCATION,
                format!("/owners/{}", owner_with_pets[0].0.id),
            ))
            .finish();

        return Ok(res);
    }

    todo!()
}
