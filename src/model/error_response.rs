use actix_web::{HttpRequest, HttpResponse};
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn handle_error(req: &HttpRequest, err: Box<dyn std::error::Error>) -> HttpResponse {
        let error_response = ErrorResponse::new(err.to_string(), req.uri().to_string());
        HttpResponse::InternalServerError().json(error_response)
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, http::StatusCode, test};
    use sea_orm::{RuntimeErr, SqlxError};

    use super::ErrorResponse;

    #[test]
    async fn test_handle_error() {
        let column_name = "test_column_name";
        let db_err = sea_orm::DbErr::Query(RuntimeErr::SqlxError(SqlxError::ColumnNotFound(
            String::from(column_name),
        )));
        let req = test::TestRequest::get().uri("/vets").to_http_request();
        let res = ErrorResponse::handle_error(&req, Box::new(db_err));
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = to_bytes(res.into_body()).await.unwrap();
        let body_json: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(
            body_json.message,
            format!("Query Error: no column found for name: {column_name}")
        );
    }
}
