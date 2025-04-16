use api_subs::models::sub::SubscriptionPlan;
use middleware::{global::GlobalLimiter, user::UserRateLimiter};

pub mod middleware {
    pub mod global;
    pub mod user;
}

pub fn global_middleware(permits_per_second: u32) -> GlobalLimiter {
    GlobalLimiter::new(permits_per_second)
}

pub fn user_middleware(plans: Vec<SubscriptionPlan>) -> UserRateLimiter {
    UserRateLimiter::new(plans)
}
