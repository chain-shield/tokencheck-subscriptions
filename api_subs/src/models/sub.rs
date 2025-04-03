use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub currency: String,
    pub interval: String,
    pub active: bool,
    pub features: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubscription {
    pub id: String,
    pub customer_id: String,
    pub price_id: String,
    pub status: String,
    pub current_period_end: i64,
    pub cancel_at_period_end: bool,
}