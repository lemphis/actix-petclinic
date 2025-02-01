use chrono::Local;
use serde::Serialize;

#[derive(Serialize)]
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
}
