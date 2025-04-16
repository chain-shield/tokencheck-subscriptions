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
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubscription {
    pub id: String,
    pub customer_id: String,
    pub sub_id: String,
    pub status: String,
    pub current_period_end: i64,
    pub cancel_at_period_end: bool,
}

// Stripe forces metadata fields to be strings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub daily_api_limit: String,
    pub monthly_api_limit: String,
}