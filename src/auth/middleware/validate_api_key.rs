use std::{future::Future, pin::Pin, rc::Rc, sync::Arc};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, Ready};
use log::info;

pub struct ValidateApiKeyMiddleware {
    subs_service_api_keys: Rc<Vec<String>>,
}

impl ValidateApiKeyMiddleware {
    pub fn new(keys: Vec<String>) -> Self {
        ValidateApiKeyMiddleware {
            subs_service_api_keys: Rc::new(keys),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ValidateApiKeyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = ValidateApiKeyMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ValidateApiKeyMiddlewareService {
            service: Arc::new(service),
            subs_service_api_keys: self.subs_service_api_keys.clone(),
        })
    }
}

pub struct ValidateApiKeyMiddlewareService<S> {
    service: Arc<S>,
    subs_service_api_keys: Rc<Vec<String>>,
}

impl<S, B> Service<ServiceRequest> for ValidateApiKeyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let valid_api_keys = self.subs_service_api_keys.clone();
        let header_key = req.headers().get("X-API-Key").and_then(|v| v.to_str().ok());
        let path = req.path().to_owned();
        let srv = Arc::clone(&self.service);

        // Check API key before moving the request
        if let Some(key) = header_key {
            if valid_api_keys.contains(&key.to_string()) {
                info!("Valid API key for path {}", path);
                // Valid API key - process normally
                let fut = srv.call(req);
                return Box::pin(async move { fut.await.map(|res| res.map_into_boxed_body()) });
            }
        }

        // Determine the error message
        let error_message = if header_key.is_some() {
            "Invalid auth api key"
        } else {
            "No api key provided"
        };

        log::error!("{} for path {}", error_message, path);
        // Create the error response
        let response = HttpResponse::Unauthorized()
            .json(serde_json::json!({ "error": error_message }))
            .map_into_boxed_body();

        // If we reach here, authentication failed
        // Extract parts before creating error response
        let (request, _payload) = req.into_parts();

        // Return the error response
        Box::pin(async move { Ok(ServiceResponse::new(request, response)) })
    }
}
