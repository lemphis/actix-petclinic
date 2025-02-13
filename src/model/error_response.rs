use actix_web::{HttpRequest, HttpResponse};
use chrono::Local;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    message: String,
    path: String,
    timestamp: String,
}

impl ErrorResponse {
    pub fn new(message: String, path: String) -> Self {
        ErrorResponse {
            message,
            path,
            timestamp: Local::now().to_rfc3339(),
        }
    }

    pub fn handle_db_error(req: &HttpRequest, db_err: &sea_orm::DbErr) -> HttpResponse {
        HttpResponse::InternalServerError().json(ErrorResponse::new(
            db_err.to_string(),
            req.uri().to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test};
    use sea_orm::{RuntimeErr, SqlxError};

    use super::ErrorResponse;

    #[test]
    async fn test_handle_db_error() {
        let column_name = "test_column_name";
        let db_err = sea_orm::DbErr::Query(RuntimeErr::SqlxError(SqlxError::ColumnNotFound(
            String::from(column_name),
        )));
        let req = test::TestRequest::get().uri("/vets").to_http_request();
        let res = ErrorResponse::handle_db_error(&req, &db_err);
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            db_err.to_string(),
            format!("Query Error: no column found for name: {column_name}")
        );
    }
}
