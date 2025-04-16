use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::types::JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_encrypted: String,
    pub name: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub permissions: JsonValue,
}
