use std::{future::Future, pin::Pin, rc::Rc, sync::Arc};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, Ready};

use crate::services::{self, auth_client::AuthClient};

pub struct AuthMiddleware {
    auth_service_url: Rc<String>,
    auth_api_key: Rc<String>,
}

impl AuthMiddleware {
    pub fn new(service_url: String, api_key: String) -> Self {
        AuthMiddleware {
            auth_service_url: Rc::new(service_url),
            auth_api_key: Rc::new(api_key),
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
            auth_service_url: self.auth_service_url.clone(),
            api_key: self.auth_api_key.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Arc<S>,
    auth_service_url: Rc<String>,
    api_key: Rc<String>,
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
        let auth_service_url = self.auth_service_url.clone();
        let api_key = self.api_key.clone();

        if !path.contains("/api/secured") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_boxed_body()) });
        }

        let token_value = req
            .headers()
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| {
                if header.starts_with("Bearer ") {
                    Some(header[7..].to_string())
                } else {
                    None
                }
            });

        // Dereference Rc<String> to String for AuthClient::new
        let auth_client = AuthClient::new(
            auth_service_url.as_ref().to_string(), // Convert Rc<String> to String
            api_key.as_ref().to_string(),          // Convert Rc<String> to String
        );

        let srv = Arc::clone(&self.service);

        Box::pin(async move {
            if let Some(token) = token_value {
                // validate token and insert claims to request object for future usage
                match auth_client.validate_token(&token).await {
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
