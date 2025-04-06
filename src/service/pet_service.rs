use sea_orm::{prelude::Date, ActiveModelTrait, ActiveValue, DbConn, EntityTrait};

use crate::{
    domain::owner::{pet, types},
    model::app_error::AppError,
};

pub struct PetService;

impl PetService {
    pub async fn fetch_all_pet_types(conn: &DbConn) -> Result<Vec<types::Model>, AppError> {
        let pet_types = types::Entity::find().all(conn).await?;

        Ok(pet_types)
    }

    pub async fn save_pet(
        conn: &DbConn,
        name: Option<String>,
        birth_date: Option<Date>,
        type_id: u32,
        owner_id: Option<u32>,
    ) -> Result<pet::Model, AppError> {
        let pet_active_model = pet::ActiveModel {
            name: ActiveValue::Set(name),
            birth_date: ActiveValue::Set(birth_date),
            type_id: ActiveValue::Set(type_id),
            owner_id: ActiveValue::Set(owner_id),
            ..Default::default()
        };

        let new_owner = pet_active_model.insert(conn).await?;

        Ok(new_owner)
    }
}
