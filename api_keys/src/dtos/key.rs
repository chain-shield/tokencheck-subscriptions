use serde::{Deserialize, Serialize};
use sqlx::types::{chrono::NaiveDateTime, JsonValue};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    pub permissions: JsonValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevokeKeyRequest {
    pub key_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyListItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hashed: String,
    pub name: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub permissions: JsonValue,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    pub id: Uuid,
    pub key: String,
    pub user_id: Uuid,
    pub name: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub permissions: JsonValue,
}