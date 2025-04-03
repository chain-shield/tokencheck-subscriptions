use serde::{Deserialize, Serialize};

use crate::models::sub::{SubscriptionPlan, UserSubscription};

#[derive(Debug, Deserialize)]
pub struct SubscriptionCreateRequest {
    pub price_id: String,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct UserSubscriptionResponse {
    pub subscription: UserSubscription,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionPlansResponse {
    pub plans: Vec<SubscriptionPlan>,
}

#[derive(Debug, Deserialize)]
pub struct EnterpriseSubscriptionRequest {
    pub name: String,
    pub amount: i64,
    pub interval: String,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAutoRenewRequest {
    pub auto_renew: bool,
}
