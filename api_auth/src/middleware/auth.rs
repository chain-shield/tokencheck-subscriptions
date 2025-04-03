use std::{future::Future, pin::Pin, rc::Rc, sync::Arc};

use actix_web::{
    Error, HttpMessage, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use common::env_config::JwtConfig;
use futures::future::{Ready, ok};

use crate::services;

pub struct AuthMiddleware {
    jwt_config: Rc<JwtConfig>,
}

impl AuthMiddleware {
    pub fn new(config: JwtConfig) -> Self {
        AuthMiddleware {
            jwt_config: Rc::new(config),
        }
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
            jwt_config: self.jwt_config.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Arc<S>,
    jwt_config: Rc<JwtConfig>,
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
        // only require authorization for paths containing "/api/secured"
        let path = req.path();
        if !path.contains("/api/secured") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_boxed_body()) });
        }

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

        let jwt_config = self.jwt_config.clone();
        let srv = Arc::clone(&self.service);

        Box::pin(async move {
            if let Some(token) = auth_header {
                // validate token and insert claims to request object for future usage
                match services::auth::validate_jwt(&token, &jwt_config.secret) {
                    Ok(claims) => {
                        req.extensions_mut().insert(claims);
                        srv.call(req).await.map(|res| res.map_into_boxed_body())
                    }
                    Err(_) => {
                        let response = HttpResponse::Unauthorized()
                            .json(serde_json::json!({"error": "Invalid token"}))
                            .map_into_boxed_body();
                        Ok(req.into_response(response))
                    }
                }
            } else {
                // no token passed - 401
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({"error": "No authorization token provided"}))
                    .map_into_boxed_body();
                Ok(req.into_response(response))
            }
        })
    }
}
