use actix_web::HttpResponse;
use thiserror::Error;

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

    #[error("Too Many Requests: {0}")]
    TooManyRequests(String),

    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn to_http_response(&self) -> HttpResponse {
        let is_dev = cfg!(debug_assertions);

        let to_internal_json = |err_msg: &str| {
            if is_dev {
                serde_json::json!({ "error": err_msg })
            } else {
                serde_json::json!({ "error": "Internal server error" })
            }
        };

        match self {
            // === CONVERSION ERRORS ===
            AppError::Database(error) => {
                log::error!("Database error: {}", error);
                HttpResponse::InternalServerError().json(to_internal_json(&error.to_string()))
            }
            AppError::JWT(error) => {
                log::error!("JWT error: {}", error);
                HttpResponse::InternalServerError().json(to_internal_json(&error.to_string()))
            }
            AppError::Reqwest(error) => {
                log::error!("Reqwest error: {}", error);
                HttpResponse::InternalServerError().json(to_internal_json(&error.to_string()))
            }
            AppError::Stripe(error) => {
                log::error!("Stripe error: {}", error);
                HttpResponse::InternalServerError().json(to_internal_json(&error.to_string()))
            }

            // === APPLICATION ERRORS ===
            AppError::Unauthorized(_) => {
                HttpResponse::Unauthorized().json(serde_json::json!({ "error": self.to_string() }))
            }
            AppError::Forbidden(_) => {
                HttpResponse::Forbidden().json(serde_json::json!({ "error": self.to_string() }))
            }
            AppError::NotFound(_) => {
                HttpResponse::NotFound().json(serde_json::json!({ "error": self.to_string() }))
            }
            AppError::BadRequest(_) => {
                HttpResponse::BadRequest().json(serde_json::json!({ "error": self.to_string() }))
            }
            AppError::TooManyRequests(_) => HttpResponse::TooManyRequests()
                .json(serde_json::json!({ "error": self.to_string() })),

            AppError::Internal(error) => {
                log::error!("Internal error: {}", error);
                HttpResponse::InternalServerError().json(to_internal_json(&error.to_string()))
            }
        }
    }
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        self.to_http_response()
    }
}
