use std::{future::Future, pin::Pin, sync::Arc};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage
};
use common::jwt;
use futures::future::{Ready, ok};

pub struct AuthMiddleware {}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Arc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Arc::clone(&self.service);

        Box::pin(async move {
            match jwt::get_jwt_claims_or_error(&req) {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                    return srv.call(req).await.map(|res| res.map_into_boxed_body())
                },
                Err(response) => return Ok(req.into_response(response)),
            }
        })
    }
}
