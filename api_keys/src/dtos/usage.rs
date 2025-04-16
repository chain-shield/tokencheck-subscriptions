use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyUsageRequest {
    pub user_id: Option<Uuid>,
    pub key_id: Option<Uuid>,
    pub limit: i32,
    pub ending_before: Option<String>,
    pub starting_after: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageResponse {
    pub name: String,
    pub date: NaiveDateTime,
    pub path: String,
}
