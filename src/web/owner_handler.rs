use std::{collections::HashMap, sync::LazyLock};

use actix_web::{
    get,
    http::{self},
    post, web, HttpRequest, HttpResponse,
};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages, Level};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tera::Context;
use validator::Validate;

use crate::{
    model::{app_error::AppError, page::Page},
    service::owner_service::OwnerService,
    web::render,
    AppState,
};

static PHONE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{10}$").unwrap());
static NUMERIC_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:\d+)?$").unwrap());

#[get("/owners/{owner_id:\\d+}")]
pub async fn show_owner(
    app_state: web::Data<AppState>,
    path: web::Path<u32>,
    messages: IncomingFlashMessages,
) -> Result<HttpResponse, AppError> {
    let owner_id = path.into_inner();
    let conn = &app_state.conn;

    let owner_with_pets_and_types =
        OwnerService::fetch_owner_with_pets_and_types_by_owner_id(conn, owner_id).await?;

    let (success_message, error_message) = extract_flash_messages(&messages);

    let mut ctx = Context::new();
    ctx.insert("owner_with_pets", &owner_with_pets_and_types);
    ctx.insert("success_message", &success_message);
    ctx.insert("error_message", &error_message);
    ctx.insert("current_menu", "owners");

    render(&app_state.tera, "owner/owner-details.html", ctx)
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
struct CreateOrUpdateOwnerForm {
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
    #[validate(regex(path = *NUMERIC_REGEX))]
    id: String,
}

#[post("/owners/new")]
pub async fn process_creation_form(
    app_state: web::Data<AppState>,
    form: web::Form<CreateOrUpdateOwnerForm>,
) -> Result<HttpResponse, AppError> {
    let owner = form.into_inner();

    if let Err(errors) = owner.validate() {
        return handle_validation_errors(&app_state.tera, owner, errors);
    }

    let new_owner = OwnerService::save_owner(
        &app_state.conn,
        Some(owner.first_name),
        Some(owner.last_name),
        Some(owner.address),
        Some(owner.city),
        Some(owner.telephone),
    )
    .await?;

    FlashMessage::info("New Owner Created").send();

    Ok(redirect_to_owner_detail(new_owner.id))
}

fn handle_validation_errors(
    tera: &tera::Tera,
    owner: CreateOrUpdateOwnerForm,
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

fn redirect_to_owner_detail(owner_id: u32) -> HttpResponse {
    HttpResponse::Found()
        .append_header((http::header::LOCATION, format!("/owners/{owner_id}")))
        .finish()
}

#[get("/owners/find")]
pub async fn init_find_form(app_state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let mut ctx = Context::new();
    ctx.insert("current_menu", "owners");

    render(&app_state.tera, "owner/find-owners.html", ctx)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct FindOwnerRequestQueryParams {
    last_name: Option<String>,
    page: Option<u64>,
    size: Option<u64>,
}

#[get("/owners")]
pub async fn process_find_form(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    query: web::Query<FindOwnerRequestQueryParams>,
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

    let owner_total_count =
        OwnerService::fetch_owner_count_by_last_name_prefix(&app_state.conn, &last_name).await?;

    if owner_total_count == 0 {
        let translation = app_state.i18n.get(&req);
        ctx.insert("translation", translation);
        ctx.insert("last_name", &last_name);

        return render(&app_state.tera, "owner/find-owners.html", ctx);
    }

    let owners_with_pet_names =
        OwnerService::fetch_owners_with_pet_names(conn, &last_name, cur_page, size).await?;

    if cur_page == 1 && owners_with_pet_names.len() == 1 {
        return Ok(redirect_to_owner_detail(owners_with_pet_names[0].id));
    }

    let page = Page::new(cur_page, owner_total_count);
    let mut ctx = Context::new();
    ctx.insert("owners", &owners_with_pet_names);
    ctx.insert("last_name", &last_name);
    ctx.insert("page", &cur_page);
    ctx.insert("total_pages", &page.total_pages());
    ctx.insert("has_previous", &page.has_previous());
    ctx.insert("has_next", &page.has_next());
    ctx.insert("page_range", page.page_range());
    ctx.insert("query_params", &vec![("last_name", last_name)]);
    ctx.insert("current_menu", "owners");

    render(&app_state.tera, "owner/owners-list.html", ctx)
}

#[get("/owners/{owner_id:\\d+}/edit")]
pub async fn init_update_owner_form(
    app_state: web::Data<AppState>,
    path: web::Path<u32>,
) -> Result<HttpResponse, AppError> {
    let owner_id = path.into_inner();
    let owner = OwnerService::fetch_owner_by_id(&app_state.conn, owner_id).await?;

    let mut ctx = Context::new();
    ctx.insert("owner", &owner);
    ctx.insert("current_menu", "owners");

    render(
        &app_state.tera,
        "owner/create-or-update-owner-form.html",
        ctx,
    )
}

#[post("/owners/{owner_id:\\d+}/edit")]
pub async fn process_update_owner_form(
    app_state: web::Data<AppState>,
    path: web::Path<u32>,
    form: web::Form<CreateOrUpdateOwnerForm>,
) -> Result<HttpResponse, AppError> {
    let owner_id = path.into_inner();
    let owner = form.into_inner();

    if let Err(errors) = owner.validate() {
        return handle_validation_errors(&app_state.tera, owner, errors);
    }

    let body_owner_id = owner.id.parse::<u32>().unwrap();
    if body_owner_id != owner_id {
        return Err(AppError::ResourceIdMismatch {
            resource: "owner".to_string(),
            path_id: owner_id,
            body_id: body_owner_id,
        });
    }

    OwnerService::update_owner(
        &app_state.conn,
        owner_id,
        Some(owner.first_name),
        Some(owner.last_name),
        Some(owner.address),
        Some(owner.city),
        Some(owner.telephone),
    )
    .await?;

    FlashMessage::info("Owner Values Updated").send();

    Ok(redirect_to_owner_detail(owner_id))
}
