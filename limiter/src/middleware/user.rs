use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use api_subs::models::sub::SubscriptionPlan;
use common::{
    error::AppError,
    key::{self},
};
use dashmap::DashMap;
use governor::{Quota, RateLimiter, clock::QuantaClock, state::keyed::DashMapStateStore};
use std::{future::Future, num::NonZeroU32, pin::Pin, rc::Rc, sync::Arc, time::Duration};
use uuid::Uuid;

type UserStateStore = DashMapStateStore<Uuid>;

pub struct UserRateLimiter {
    plans: Vec<SubscriptionPlan>,
}

impl UserRateLimiter {
    pub fn new(plans: Vec<SubscriptionPlan>) -> Self {
        Self { plans }
    }
}

impl<S, B> Transform<S, ServiceRequest> for UserRateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = UserRateLimiterService<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let daily_limiters = DashMap::new();
        let monthly_limiters = DashMap::new();
        for plan in &self.plans {
            if let Some(metadata) = &plan.metadata {
                // Parse daily limit
                let daily_limit = match metadata.daily_api_limit.parse::<u32>() {
                    Ok(val) => match NonZeroU32::new(val) {
                        Some(nonzero) => nonzero,
                        None => {
                            log::error!("Daily limit is zero for plan {}", plan.id);
                            continue;
                        }
                    },
                    Err(e) => {
                        log::error!(
                            "Failed to parse daily_api_limit for plan {}: {}",
                            plan.id,
                            e
                        );
                        continue;
                    }
                };

                // Parse monthly limit
                let monthly_limit = match metadata.monthly_api_limit.parse::<u32>() {
                    Ok(val) => match NonZeroU32::new(val) {
                        Some(nonzero) => nonzero,
                        None => {
                            log::error!("Monthly limit is zero for plan {}", plan.id);
                            continue;
                        }
                    },
                    Err(e) => {
                        log::error!(
                            "Failed to parse monthly_api_limit for plan {}: {}",
                            plan.id,
                            e
                        );
                        continue;
                    }
                };

                // Create daily quota
                let daily_quota = match Quota::with_period(Duration::from_secs((24 * 60 * 60 / daily_limit) as u64)) {
                    Some(q) => q.allow_burst(daily_limit),
                    None => {
                        log::error!("Failed to create daily quota for plan {}", plan.id);
                        continue;
                    }
                };

                // Create monthly quota
                let monthly_quota = match Quota::with_period(Duration::from_secs((30 * 24 * 60 * 60 / monthly_limit) as u64))
                {
                    Some(q) => q.allow_burst(monthly_limit),
                    None => {
                        log::error!("Failed to create monthly quota for plan {}", plan.id);
                        continue;
                    }
                };

                daily_limiters.insert(plan.id.clone(), Arc::new(RateLimiter::keyed(daily_quota)));
                monthly_limiters
                    .insert(plan.id.clone(), Arc::new(RateLimiter::keyed(monthly_quota)));
            } else {
                log::error!("Limits not available for plan {}", plan.id);
            }
        }

        std::future::ready(Ok(UserRateLimiterService {
            service: Rc::new(service),
            daily_limiters,
            monthly_limiters,
        }))
    }
}

pub struct UserRateLimiterService<S> {
    service: Rc<S>,
    pub daily_limiters: DashMap<String, Arc<RateLimiter<Uuid, UserStateStore, QuantaClock>>>,
    pub monthly_limiters: DashMap<String, Arc<RateLimiter<Uuid, UserStateStore, QuantaClock>>>,
}

impl<S, B> Service<ServiceRequest> for UserRateLimiterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);
        let daily_limiters = self.daily_limiters.clone();
        let monthly_limiters = self.monthly_limiters.clone();

        Box::pin(async move {
            // Check if request contains API key
            if let Some(key_claims) = key::get_key_claims_or_error(&req).ok() {
                // Get info
                let user_id = key_claims.user_id;
                let plan_id = key_claims.plan_id;

                // Check daily limits
                let daily_limiter_opt = daily_limiters.get(&plan_id);
                if let Some(daily_limiter) = daily_limiter_opt {
                    match daily_limiter.check_key(&user_id) {
                        Ok(_) => {}
                        Err(_) => {
                            return Ok(req.error_response(AppError::TooManyRequests(
                                "You have exceeded daily limit for your plan".to_string(),
                            )));
                        }
                    }
                } else {
                    log::error!("Failed to find daily limiter for plan {}", plan_id);
                }

                // Check monthly limits
                let monthly_limiter_opt = monthly_limiters.get(&plan_id);
                if let Some(monthly_limiter) = monthly_limiter_opt {
                    match monthly_limiter.check_key(&user_id) {
                        Ok(_) => {}
                        Err(_) => {
                            return Ok(req.error_response(AppError::TooManyRequests(
                                "You have exceeded monthly limit for your plan".to_string(),
                            )));
                        }
                    }
                } else {
                    log::error!("Failed to find monthly limiter for plan {}", plan_id);
                }
            }

            srv.call(req).await.map(|res| res.map_into_boxed_body())
        })
    }
}
