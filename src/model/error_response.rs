use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    message: String,
    timestamp: String,
}

impl ErrorResponse {
    pub fn new(message: String) -> Self {
        ErrorResponse {
            message,
            timestamp: Local::now().to_rfc3339(),
        }
    }
}
