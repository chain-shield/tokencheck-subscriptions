use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct OneTimeResponse {
    pub url: String,
}

#[derive(Deserialize)]
pub struct SubscriptionRequest {
    pub price_id: String,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Deserialize)]
pub struct RecurringInfo {
    pub interval: String,
    pub interval_count: u64,
}
#[derive(Deserialize)]
pub struct CustomSubscriptionRequest {
    pub product_id: String,
    pub amount: i64,
    pub recurring_info: Option<RecurringInfo>,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Deserialize)]
pub struct RefundRequest {
    pub payment_intent_id: String,
    pub amount: Option<i64>,    // Optional: partial refund amount in cents
    pub reason: Option<String>, // Optional: reason for refund
}

#[derive(Serialize)]
pub struct RefundResponse {
    pub id: String,
    pub amount: i64,
    pub status: String,
    pub payment_intent_id: String,
}

#[derive(Deserialize)]
pub struct PaymentIntentsRequest {
    pub user_id: Option<String>, // Optional: Get intents for a specific user (if null, use authenticated user)
    pub limit: Option<u64>,      // Optional: Limit number of results
    pub ending_before: Option<String>, // Optional: Cursor for pagination (exclusive)
    pub starting_after: Option<String>, // Optional: Cursor for pagination (exclusive)
}

#[derive(Serialize)]
pub struct PaymentIntent {
    pub id: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub created: i64,
}
#[derive(Serialize)]
pub struct PaymentIntentsResponse {
    pub intents: Vec<PaymentIntent>,
}

