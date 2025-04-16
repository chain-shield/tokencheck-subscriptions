use actix_web::{HttpMessage, HttpResponse, dev::ServiceRequest};
use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, Res};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyClaims {
    pub user_id: Uuid,
    pub plan_id: String,
    pub key_id: Uuid,
    pub secret: String,
}

impl KeyClaims {
    pub fn to_key(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        let encoded = general_purpose::STANDARD.encode(json);
        format!("sk_{}", encoded)
    }

    pub fn from_key(key: &str) -> Res<Self> {
        let encoded = key
            .strip_prefix("sk_")
            .ok_or_else(|| AppError::BadRequest("Missing prefix 'sk_'".to_string()))?;

        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| AppError::BadRequest(format!("Base64 decode error: {}", e)))?;

        let claims = serde_json::from_slice(&decoded_bytes)
            .map_err(|e| AppError::BadRequest(format!("JSON parse error: {}", e)))?;

        Ok(claims)
    }
}

pub fn get_key_claims_or_error(req: &ServiceRequest) -> Result<KeyClaims, HttpResponse> {
    if let Some(key_claims_res) = req.extensions().get::<Res<KeyClaims>>() {
        match key_claims_res {
            Ok(claims) => Ok(claims.clone()),
            Err(app_error) => Err(app_error.to_http_response()),
        }
    } else {
        Err(AppError::Unauthorized("No API key provided".to_string()).to_http_response())
    }
}
