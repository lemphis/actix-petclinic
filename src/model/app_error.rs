use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

use super::error_response::ErrorResponse;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Resource not found: {resource} with id: {id}")]
    ResourceNotFound { resource: String, id: u32 },

    #[error(
        "Resource ID mismatch: {resource} - Path ID {path_id} does not match Body ID {body_id}"
    )]
    ResourceIdMismatch {
        resource: String,
        path_id: u32,
        body_id: u32,
    },

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
            AppError::ResourceNotFound { .. } => HttpResponse::NotFound(),
            AppError::ResourceIdMismatch { .. } => HttpResponse::BadRequest(),
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
