use std::{future::Future, pin::Pin, sync::Arc};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, web, Error, HttpMessage
};
use futures::future::{Ready, ok};

use common::{
    env_config::Config, error::Res, jwt::{self, JwtClaims}, key::{self, KeyClaims}
};

pub struct ExtractionMiddleware {}

impl ExtractionMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, B> Transform<S, ServiceRequest> for ExtractionMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = ExtractionMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ExtractionMiddlewareService {
            service: Arc::new(service),
        })
    }
}

pub struct ExtractionMiddlewareService<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for ExtractionMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // retrieve token from authorization header
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|auth_value| {
                if auth_value.starts_with("Bearer ") {
                    Some(auth_value[7..].to_owned())
                } else {
                    None
                }
            });
        // retrieve API key from "X-API-KEY" header
        let api_key = req
            .headers()
            .get("X-API-KEY")
            .map(|v| v.to_str().unwrap_or_default().to_string());

        let config = &***req.app_data::<web::Data<Arc<Config>>>().unwrap().clone();
        let jwt_config = config.jwt_config.clone();
        let srv = Arc::clone(&self.service);

        Box::pin(async move {
            if let Some(token) = auth_header {
                // validate token and insert claims to request object for future use
                let claims_res = jwt::validate_jwt(&token, &jwt_config.secret);
                req.extensions_mut().insert::<Res<JwtClaims>>(claims_res);
            }
            if let Some(key) = api_key {
                // parse the api key and insert claims to request object for future use
                let claims_res = key::KeyClaims::from_key(key.as_str());
                req.extensions_mut().insert::<Res<KeyClaims>>(claims_res);
            }
            srv.call(req).await.map(|res| res.map_into_boxed_body())
        })
    }
}
