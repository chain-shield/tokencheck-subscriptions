use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use common::error::AppError;
use governor::{
    Quota, RateLimiter,
    clock::QuantaClock,
    state::{InMemoryState, NotKeyed},
};
use std::{future::Future, num::NonZeroU32, pin::Pin, rc::Rc, sync::Arc};

/// This limiter works for each request coming in (not per user)
pub struct GlobalLimiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, QuantaClock>>,
}

impl GlobalLimiter {
    pub fn new(permits_per_sec: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(permits_per_sec).unwrap());
        let limiter = Arc::new(RateLimiter::direct(quota));
        Self { limiter }
    }
}

impl<S, B> Transform<S, ServiceRequest> for GlobalLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = GlobalLimiterService<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(GlobalLimiterService {
            service: Rc::new(service),
            limiter: self.limiter.clone(),
        }))
    }
}

pub struct GlobalLimiterService<S> {
    service: Rc<S>,
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, QuantaClock>>,
}

impl<S, B> Service<ServiceRequest> for GlobalLimiterService<S>
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
        let limiter = self.limiter.clone();
        Box::pin(async move {
            // Check if the rate limiter allows the request
            if limiter.check().is_ok() {
                // Move to the next services if ok
                srv.call(req).await.map(|res| res.map_into_boxed_body())
            } else {
                // Return 429 if limit reached
                Ok(req.error_response(AppError::TooManyRequests(
                    "Server overloaded. Please try again later.".to_string(),
                )))
            }
        })
    }
}
