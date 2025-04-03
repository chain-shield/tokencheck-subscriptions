use std::net::IpAddr;

use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;
use sqlx::types::JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hashed: String,
    pub name: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub last_used: Option<NaiveDateTime>,
    pub permissions: JsonValue,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct ApiUsage {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub timestamp: NaiveDateTime,
    pub endpoint: String,
    pub query_params: Option<JsonValue>,
    pub request_body: Option<JsonValue>,
    pub response_body: Option<JsonValue>,
    pub response_code: i32,
    pub ip_address: IpAddr,
    pub user_agent: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct ApiUsageDaily {
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub date: NaiveDate,
    pub call_count: i32,
    pub successful_count: i32,
    pub failed_count: i32,
    pub remaining_daily_count: i32,
}
