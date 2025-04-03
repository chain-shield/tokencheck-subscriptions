use actix_web::HttpResponse;
use thiserror::Error;
use log::error;

pub type Res<T> = std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    // === CONVERSION ERRORS ===
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Stripe error: {0}")]
    Stripe(#[from] stripe::StripeError),

    // === APPLICATION ERRORS ===
    #[error("Authorization error: {0}")]
    Unauthorized(String),

    #[error("Resource conflict: {0}")]
    Forbidden(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn to_http_response(&self) -> HttpResponse {
        let json_response = serde_json::json!({"error": self.to_string()});

        match self {
            // === CONVERSION ERRORS ===
            AppError::Database(error) => {
                error!("Database error: {}", error);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Internal server error"}))
            }
            AppError::JWT(error) => {
                error!("JWT error: {}", error);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Internal server error"}))
            }
            AppError::Reqwest(error) => {
                error!("Reqwest error: {}", error);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Internal server error"}))
            }
            AppError::Stripe(error) => {
                error!("Stripe error: {}", error);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Internal server error"}))
            }

            // === APPLICATION ERRORS ===
            AppError::Unauthorized(_) => HttpResponse::Unauthorized().json(json_response),
            AppError::Forbidden(_) => HttpResponse::Forbidden().json(json_response),
            AppError::NotFound(_) => HttpResponse::NotFound().json(json_response),
            AppError::BadRequest(_) => HttpResponse::BadRequest().json(json_response),

            AppError::Internal(error) => {
                error!("Internal error: {}", error);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Internal server error"}))
            }
        }
    }
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        self.to_http_response()
    }
}
