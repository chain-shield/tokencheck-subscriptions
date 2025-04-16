use sqlx::types::JsonValue;
use uuid::Uuid;

pub struct KeyCreateRequest {
    pub user_id: Uuid,
    pub key_encrypted: String,
    pub name: String,
    pub permissions: JsonValue,
}

pub struct KeyUpdateRequest {
    pub name: Option<String>,
    pub permissions: Option<JsonValue>,
}