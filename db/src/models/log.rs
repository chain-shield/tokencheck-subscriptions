use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::types::{JsonValue, ipnetwork::IpNetwork};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Log {
    pub id: Uuid,
    pub timestamp: NaiveDateTime,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub user_id: Option<Uuid>,
    pub params: Option<JsonValue>,
    pub request_body: Option<JsonValue>,
    pub response_body: Option<JsonValue>,
    pub ip_address: IpNetwork,
    pub user_agent: String,
}
