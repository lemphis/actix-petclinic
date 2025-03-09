use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

use super::error_response::ErrorResponse;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Owner not found with id: {0}. Please ensure the ID is correct and the owner exists in the database.")]
    OwnerNotFound(u32),

    #[error("Database error: {0}")]
    DbError(#[from] sea_orm::DbErr),

    #[error("Template error: {0}")]
    TemplateError(#[from] tera::Error),

    #[error("XML serialize error: {0}")]
    SerializeError(#[from] quick_xml::SeError),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let mut res_builder = match self {
            AppError::OwnerNotFound(_) => HttpResponse::NotFound(),
            AppError::DbError(_) => HttpResponse::InternalServerError(),
            AppError::TemplateError(_) => HttpResponse::InternalServerError(),
            AppError::SerializeError(_) => HttpResponse::InternalServerError(),
        };

        let err_body = ErrorResponse::new(self.to_string());

        res_builder
            .insert_header(("App-Error", "true"))
            .json(err_body)
    }
}
