use actix_web::{HttpMessage, HttpResponse, dev::ServiceRequest};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    env_config::JwtConfig,
    error::{AppError, Res},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub user_id: Uuid,
    pub stripe_customer_id: String,
    pub exp: usize,
}

pub struct ClaimsSpec {
    pub user_id: Uuid,
    pub stripe_customer_id: String,
}

/// Generates JWT token based on user object and JWT configuration options
pub fn generate_jwt(spec: ClaimsSpec, config: &JwtConfig) -> Res<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(config.expiration_hours))
        .expect("valid timestamp")
        .timestamp();

    let claims = JwtClaims {
        user_id: spec.user_id,
        stripe_customer_id: spec.stripe_customer_id,
        exp: expiration as usize,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(AppError::from)
}

/// Extracts claims object from JWT token.
/// Requires JWT secret.
pub fn validate_jwt(token: &str, secret: &str) -> Res<JwtClaims> {
    let token_data = jsonwebtoken::decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub fn get_jwt_claims_or_error(req: &ServiceRequest) -> Result<JwtClaims, HttpResponse> {
    if let Some(jwt_claims_res) = req.extensions().get::<Res<JwtClaims>>() {
        match jwt_claims_res {
            Ok(claims) => Ok(claims.clone()),
            Err(app_error) => Err(app_error.to_http_response()),
        }
    } else {
        Err(
            AppError::Unauthorized("No authorization token provided".to_string())
                .to_http_response(),
        )
    }
}
