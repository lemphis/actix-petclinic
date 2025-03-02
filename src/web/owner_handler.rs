use std::collections::HashMap;

use actix_web::{get, http::header::ContentType, web, HttpRequest, HttpResponse, Responder};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tera::Context;

use crate::{
    domain::owner::{owners, pet, types},
    model::{app_error::AppError, error_response::ErrorResponse},
    AppState,
};

#[get("/owners/{id:\\d+}")]
pub async fn show_owner(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    path: web::Path<u32>,
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
    let pet_types_map: HashMap<u32, types::Model> =
        pet_types.into_iter().map(|pt| (pt.id, pt)).collect();
    let mut pets_with_type: Vec<(pet::Model, types::Model)> = pets
        .into_iter()
        .filter_map(|pet| {
            pet_types_map
                .get(&pet.type_id)
                .map(|pet_type| (pet, pet_type.clone()))
        })
        .collect();
    pets_with_type.sort_by(|(pet_a, _), (pet_b, _)| pet_a.name.cmp(&pet_b.name));

    let mut context = Context::new();
    context.insert("owner", &owner);
    context.insert("pets", &pets_with_type);
    context.insert("current_menu", "owners");

    match app_state.tera.render("owner/owner-details.html", &context) {
        Ok(html) => HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(html),
        Err(err) => ErrorResponse::handle_error(&req, &err),
    }
}
