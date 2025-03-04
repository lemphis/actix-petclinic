use std::collections::HashMap;

use actix_web::{get, web, HttpRequest, Responder};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tera::Context;

use crate::{
    domain::owner::{owners, pet, types},
    model::{app_error::AppError, error_response::ErrorResponse},
    web::render,
    AppState,
};

#[get("/owners/{id:\\d+}")]
pub async fn show_owner(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    path: web::Path<u32>,
    messages: IncomingFlashMessages,
) -> impl Responder {
    let owner_id = path.into_inner();
    let conn = &app_state.conn;

    let owner_with_pets = match owners::Entity::find_by_id(owner_id)
        .find_with_related(pet::Entity)
        .all(conn)
        .await
    {
        Ok(data) => data,
        Err(db_err) => return ErrorResponse::handle_error(&req, &db_err),
    };

    let (owner, pets) = if let Some(record) = owner_with_pets.into_iter().next() {
        record
    } else {
        return ErrorResponse::handle_error(&req, &AppError::OwnerNotFound(owner_id));
    };

    let pet_type_ids: Vec<u32> = pets.iter().map(|p| p.type_id).collect();
    let pet_types = match types::Entity::find()
        .filter(types::Column::Id.is_in(pet_type_ids))
        .all(conn)
        .await
    {
        Ok(data) => data,
        Err(db_err) => return ErrorResponse::handle_error(&req, &db_err),
    };

    let pets_with_type = join_pets_with_types(pets, pet_types);

    let message = messages
        .iter()
        .find(|flash_message| flash_message.level() == Level::Info)
        .map(|flash_message| flash_message.content());
    let error = messages
        .iter()
        .find(|flash_message| flash_message.level() == Level::Error)
        .map(|flash_message| flash_message.content());

    let mut context = Context::new();
    context.insert("owner", &owner);
    context.insert("pets", &pets_with_type);
    context.insert("message", &message);
    context.insert("error", &error);
    context.insert("current_menu", "owners");

    render(req, &app_state.tera, "owner/owner-details.html", context)
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
pub async fn init_creation_form(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let mut context = Context::new();
    context.insert("current_menu", "owners");

    render(
        req,
        &app_state.tera,
        "owner/create-or-update-owner-form.html",
        context,
    )
}
