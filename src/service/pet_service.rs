use sea_orm::{DbConn, EntityTrait};

use crate::{domain::owner::types, model::app_error::AppError};

pub struct PetService;

impl PetService {
    pub async fn fetch_all_pet_types(conn: &DbConn) -> Result<Vec<types::Model>, AppError> {
        let pet_types = types::Entity::find().all(conn).await?;

        Ok(pet_types)
    }
}
