use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use common::{
    error::AppError,
    key::{self},
};
use futures::future::{Ready, ok};
use sqlx::PgPool;
use std::{future::Future, pin::Pin, sync::Arc};

// KeyMiddleware struct (as a Transform)
pub struct KeyMiddleware {}

impl KeyMiddleware {
    pub fn new() -> Self {
        KeyMiddleware {}
    }
}

// Implement the Transform trait for KeyMiddleware
impl<S, B> Transform<S, ServiceRequest> for KeyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = KeyMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(KeyMiddlewareService {
            service: Arc::new(service),
        })
    }
}

// Service struct for the middleware
pub struct KeyMiddlewareService<S> {
    service: Arc<S>,
}

// Implement the Service trait for KeyMiddlewareService
impl<S, B> Service<ServiceRequest> for KeyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Arc::clone(&self.service);

        Box::pin(async move {
            let pool = &***req.app_data::<web::Data<Arc<PgPool>>>().unwrap().clone();
            // Extract key claims from the request
            match key::get_key_claims_or_error(&req) {
                Err(response) => {
                    return Ok(req.into_response(response));
                }
                Ok(key_claims) => {
                    // fetch record from database
                    match db::key::get_key_by_id(pool, &key_claims.key_id).await {
                        Ok(key_record) => {
                            // check if secret matches hashed value from database
                            let parsed_hash = PasswordHash::new(&key_record.key_encrypted).unwrap();
                            if Argon2::default()
                                .verify_password(key_claims.secret.as_bytes(), &parsed_hash)
                                .is_err()
                            {
                                return Ok(req.error_response(AppError::BadRequest(
                                    "Invalid key".to_string(),
                                )));
                            }

                            // ... optional permissions check here ...

                            srv.call(req).await.map(|res| res.map_into_boxed_body())
                        }
                        Err(_) => {
                            Ok(req.error_response(AppError::BadRequest("Invalid key".to_string())))
                        }
                    }
                }
            }
        })
    }
}
