use api_subs::models::sub::SubscriptionPlan;
use middleware::{global::GlobalLimiter, quota::QuotaRateLimiter};

pub mod middleware {
    pub mod global;
    pub mod quota;
}

pub fn global_middleware(permits_per_second: u32) -> GlobalLimiter {
    GlobalLimiter::new(permits_per_second)
}

pub fn quota_middleware(
    plans: Vec<SubscriptionPlan>,
    redis_client: redis::Client,
) -> QuotaRateLimiter {
    QuotaRateLimiter::new(plans, redis_client)
}
