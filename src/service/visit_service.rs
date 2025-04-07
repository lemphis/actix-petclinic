use sea_orm::{prelude::Date, ActiveModelTrait, ActiveValue, DbConn};

use crate::{domain::owner::visit, model::app_error::AppError};

pub struct VisitService;

impl VisitService {
    pub async fn save_visit(
        conn: &DbConn,
        pet_id: Option<u32>,
        visit_date: Option<Date>,
        description: Option<String>,
    ) -> Result<visit::Model, AppError> {
        let visit_active_model = visit::ActiveModel {
            pet_id: ActiveValue::Set(pet_id),
            visit_date: ActiveValue::Set(visit_date),
            description: ActiveValue::Set(description),
            ..Default::default()
        };

        let new_owner = visit_active_model.insert(conn).await?;

        Ok(new_owner)
    }
}
