use std::{collections::HashMap, rc::Rc, sync::Arc};

use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use api_subs::models::sub::SubscriptionPlan;

use ::chrono::{/* Datelike, */ Duration};
use chrono::{/* NaiveDate, */ Utc};
use common::{
    error::AppError,
    key::{self},
};
use redis::AsyncCommands;
use sqlx::types::chrono;
use std::{future::Future, pin::Pin};

// --- Rate Limiting Middleware Definition ---

pub struct QuotaRateLimiter {
    plans: Arc<HashMap<String, Arc<SubscriptionPlan>>>,
    redis_client: Arc<redis::Client>,
}

impl QuotaRateLimiter {
    pub fn new(plans_vec: Vec<SubscriptionPlan>, redis_client: redis::Client) -> Self {
        let mut plans_map = HashMap::new();
        for plan in plans_vec {
            plans_map.insert(plan.id.clone(), Arc::new(plan));
        }
        QuotaRateLimiter {
            plans: Arc::new(plans_map),
            redis_client: Arc::new(redis_client),
        }
    }
}

// --- Middleware Transform Implementation ---

impl<S, B> Transform<S, ServiceRequest> for QuotaRateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = QuotaRateLimitingMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(QuotaRateLimitingMiddleware {
            service: Rc::new(service),
            plans: Arc::clone(&self.plans),
            redis_client: Arc::clone(&self.redis_client),
        }))
    }
}

// --- Actual Middleware Service ---

pub struct QuotaRateLimitingMiddleware<S> {
    // Use Rc for the service within a single worker thread
    service: Rc<S>,
    plans: Arc<HashMap<String, Arc<SubscriptionPlan>>>,
    redis_client: Arc<redis::Client>,
}

// --- Service Trait Implementation for the Middleware ---

impl<S, B> Service<ServiceRequest> for QuotaRateLimitingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    // Readiness check - forward to the inner service
    forward_ready!(service);

    // Main logic: This method is called for each request
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Clone Arcs to move into the async block
        let plans = Arc::clone(&self.plans);
        let redis_client = Arc::clone(&self.redis_client);
        // Clone Rc for the service
        let srv = Rc::clone(&self.service);

        Box::pin(async move {
            // Check if API key claims are present in the request
            if let Some(key_claims) = key::get_key_claims_or_error(&req).ok() {
                // 1. Find the subscription plan based on claims
                let plan = match plans.get(&key_claims.plan_id) {
                    Some(p) => Arc::clone(p), // Clone the Arc<SubscriptionPlan>
                    None => {
                        return Ok(req.error_response(AppError::Internal(format!(
                            "Plan ID '{}' from key '{}' not found in configured plans.",
                            key_claims.plan_id, key_claims.key_id
                        ))));
                    }
                };

                // 2. Parse limits from metadata
                let (daily_limit, monthly_limit) = match &plan.metadata {
                    Some(meta) => {
                        match (
                            meta.daily_api_limit.parse::<u64>(),
                            meta.monthly_api_limit.parse::<u64>(),
                        ) {
                            (Ok(d), Ok(m)) => (d, m),
                            _ => {
                                return Ok(req.error_response(AppError::Internal(format!(
                                    "Failed to parse limits for plan ID '{}'",
                                    key_claims.plan_id
                                ))));
                            }
                        }
                    }
                    None => {
                        log::warn!(
                            "Plan ID '{}' has no metadata defined. Allowing request without limits.",
                            key_claims.plan_id
                        );
                        // If no limits defined, allow the request to pass through
                        return srv.call(req).await.map(|res| res.map_into_boxed_body());
                    }
                };

                // Skip checks if limits are zero (or handle as needed)
                if daily_limit == 0 || monthly_limit == 0 {
                    log::debug!(
                        "Plan '{}' has zero limits, allowing request.",
                        key_claims.plan_id
                    );
                    return srv.call(req).await.map(|res| res.map_into_boxed_body());
                }

                // 3. Get Redis connection
                let mut redis_conn = match redis_client.get_multiplexed_tokio_connection().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        return Ok(req.error_response(AppError::Internal(format!(
                            "Failed to get Redis connection: {}",
                            e
                        ))));
                    }
                };

                // 4. Prepare Redis keys and TTLs
                let now = Utc::now();
                let date_str = now.format("%Y-%m-%d").to_string();
                let month_str = now.format("%Y-%m").to_string();
                let user_id_str = key_claims.user_id.to_string();

                // Create Redis keys for daily and monthly quotas
                let daily_key = format!("quota:{}:daily:{}", user_id_str, date_str);
                let monthly_key = format!("quota:{}:monthly:{}", user_id_str, month_str);

                // Calculate TTLs for daily and monthly quotas
                let seconds_until_midnight = calculate_seconds_until_midnight(now);
                // let seconds_until_end_of_month = calculate_seconds_until_end_of_month(now);

                // 5. Check and Increment Limits (Atomic Check-then-Increment is tricky without Lua)
                // Approach: Increment first, check result, decrement if over limit.

                // --- Daily Check ---
                // Increment the daily count in Redis
                let daily_count: Result<u64, redis::RedisError> =
                    redis_conn.incr(&daily_key, 1).await;

                match daily_count {
                    Ok(count) => {
                        // Set expiry only if the key was newly created (count is 1)
                        if count == 1 {
                            let _: Result<(), redis::RedisError> = redis_conn
                                .expire(&daily_key, seconds_until_midnight as i64)
                                .await;
                        }

                        // Check if daily limit is exceeded
                        if count > daily_limit {
                            // Decrement back as we exceeded the limit
                            let _: Result<u64, redis::RedisError> =
                                redis_conn.decr(&daily_key, 1).await;

                            return Ok(req.error_response(AppError::TooManyRequests(format!(
                                "Daily limit exceeded for key {}. Count: {}, Limit: {}",
                                user_id_str, count, daily_limit
                            ))));
                        }
                    }
                    Err(e) => {
                        return Ok(req.error_response(AppError::Internal(format!(
                            "Redis error incrementing daily count for key {}: {}",
                            user_id_str, e
                        ))));
                    }
                }

                // --- Monthly Check ---
                // Increment the monthly count in Redis
                let monthly_count: Result<u64, redis::RedisError> =
                    redis_conn.incr(&monthly_key, 1).await;

                // match monthly_count {
                //     Ok(count) => {
                //         // Set expiry only if the key was newly created (count is 1)
                //         if count == 1 {
                //             let _: Result<(), redis::RedisError> = redis_conn
                //                 .expire(&monthly_key, seconds_until_end_of_month as i64)
                //                 .await;
                //         }

                //         // Check if monthly limit is exceeded
                //         if count > monthly_limit {
                //             // Decrement back BOTH counters as this request is fully rejected
                //             let _: Result<u64, redis::RedisError> =
                //                 redis_conn.decr(&monthly_key, 1).await;
                //             let _: Result<u64, redis::RedisError> =
                //                 redis_conn.decr(&daily_key, 1).await; // Also undo daily incr
                //             // Log decrement errors if needed

                //             return Ok(req.error_response(AppError::TooManyRequests(format!(
                //                 "Monthly limit exceeded for key {}. Count: {}, Limit: {}",
                //                 user_id_str, count, monthly_limit
                //             ))));
                //         }
                //     }
                //     Err(e) => {
                //         // We already incremented daily, attempt to decrement it back
                //         let _: Result<u64, redis::RedisError> =
                //             redis_conn.decr(&daily_key, 1).await;

                //         return Ok(req.error_response(AppError::Internal(format!(
                //             "Redis error incrementing monthly count for key {}: {}",
                //             user_id_str, e
                //         ))));
                //     }
                // }

                // 6. Limits OK - Forward request to the next service
                log::debug!(
                    "Limits OK for key {}. Daily: {}/{}, Monthly: {}/{}",
                    user_id_str,
                    daily_count.unwrap_or(0), // Should be Ok here
                    daily_limit,
                    monthly_count.unwrap_or(0), // Should be Ok here
                    monthly_limit
                );
            } else {
                log::warn!("No API key provided and QuotaRateLimiter was requested");
            }

            srv.call(req).await.map(|res| res.map_into_boxed_body())
        })
    }
}

// --- Helper Functions ---

fn calculate_seconds_until_midnight(now: chrono::DateTime<Utc>) -> u64 {
    let midnight_tomorrow = (now.date_naive() + Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .unwrap();

    // Ensure we are using UTC for calculation consistency
    let midnight_tomorrow_utc =
        chrono::DateTime::<Utc>::from_naive_utc_and_offset(midnight_tomorrow, Utc);

    midnight_tomorrow_utc
        .signed_duration_since(now)
        .num_seconds()
        .max(0) as u64
}

// fn calculate_seconds_until_end_of_month(now: chrono::DateTime<Utc>) -> u64 {
//     let current_month = now.month();
//     let current_year = now.year();

//     let next_month_year;
//     let next_month;

//     if current_month == 12 {
//         next_month = 1;
//         next_month_year = current_year + 1;
//     } else {
//         next_month = current_month + 1;
//         next_month_year = current_year;
//     }

//     // First day of the next month
//     let first_day_next_month = NaiveDate::from_ymd_opt(next_month_year, next_month, 1)
//         .unwrap()
//         .and_hms_opt(0, 0, 0)
//         .unwrap();

//     // Ensure we are using UTC for calculation consistency
//     let first_day_next_month_utc =
//         chrono::DateTime::<Utc>::from_naive_utc_and_offset(first_day_next_month, Utc);

//     first_day_next_month_utc
//         .signed_duration_since(now)
//         .num_seconds()
//         .max(0) as u64
// }
