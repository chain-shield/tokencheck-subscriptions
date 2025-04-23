//! # Authentication Middleware Module
//!
//! This module provides middleware for authenticating requests to secured API endpoints.
//! It validates JWT tokens by communicating with an external authentication service.
//!
//! ## Overview
//! The middleware intercepts requests to secured endpoints, extracts the JWT token from
//! the Authorization header, validates it with the authentication service, and adds the
//! validated claims to the request extensions for use by route handlers.
//!
//! ## Usage
//! ```rust
//! // In main.rs or app configuration
//! .service(
//!     web::scope("/secured")
//!         .wrap(auth::AuthMiddleware::new(
//!             config.auth_service_url.clone(),
//!             config.auth_api_key.clone()
//!         ))
//!         .service(/* secured endpoints */)
//! )
//! ```

use std::{future::Future, pin::Pin, rc::Rc, sync::Arc};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, Ready};

use crate::auth::services::auth_client::AuthClient;

/// Authentication middleware for securing API endpoints.
///
/// This middleware validates JWT tokens by communicating with an external
/// authentication service. It extracts the token from the Authorization header,
/// validates it, and adds the claims to the request extensions.
///
/// # Fields
/// * `auth_service_url` - URL of the authentication service
/// * `auth_api_key` - API key for the authentication service
pub struct AuthMiddleware {
    auth_service_url: Rc<String>,
    auth_api_key: Rc<String>,
}

impl AuthMiddleware {
    /// Creates a new instance of the authentication middleware.
    ///
    /// # Arguments
    /// * `service_url` - URL of the authentication service
    /// * `api_key` - API key for the authentication service
    ///
    /// # Returns
    /// A new instance of `AuthMiddleware`
    pub fn new(service_url: String, api_key: String) -> Self {
        AuthMiddleware {
            auth_service_url: Rc::new(service_url),
            auth_api_key: Rc::new(api_key),
        }
    }
}

/// Implementation of the `Transform` trait for `AuthMiddleware`.
///
/// This implementation creates a new middleware service that will be used
/// to process requests. It's part of Actix Web's middleware system.
///
/// # Type Parameters
/// * `S` - The service type this middleware wraps
/// * `B` - The message body type of the wrapped service
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

    /// Creates a new middleware service.
    ///
    /// # Arguments
    /// * `service` - The service to wrap
    ///
    /// # Returns
    /// A future that resolves to the middleware service
    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Arc::new(service),
            auth_service_url: self.auth_service_url.clone(),
            api_key: self.auth_api_key.clone(),
        })
    }
}

/// Service implementation for the authentication middleware.
///
/// This service is created by the `AuthMiddleware` and handles the actual
/// request processing and token validation.
///
/// # Type Parameters
/// * `S` - The service type this middleware wraps
///
/// # Fields
/// * `service` - The wrapped service
/// * `auth_service_url` - URL of the authentication service
/// * `api_key` - API key for the authentication service
pub struct AuthMiddlewareService<S> {
    service: Arc<S>,
    auth_service_url: Rc<String>,
    api_key: Rc<String>,
}

/// Implementation of the `Service` trait for `AuthMiddlewareService`.
///
/// This implementation handles the actual request processing and token validation.
/// It extracts the JWT token from the Authorization header, validates it with the
/// authentication service, and adds the validated claims to the request extensions.
///
/// # Type Parameters
/// * `S` - The service type this middleware wraps
/// * `B` - The message body type of the wrapped service
impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    // Forward the ready state of the wrapped service
    forward_ready!(service);

    /// Processes the request and validates the JWT token if required.
    ///
    /// # Arguments
    /// * `req` - The service request to process
    ///
    /// # Returns
    /// A future that resolves to the service response
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Only require authorization for paths containing "/api/secured"
        // This allows public endpoints to bypass authentication
        let path = req.path();
        let auth_service_url = self.auth_service_url.clone();
        let api_key = self.api_key.clone();

        // Skip authentication for non-secured paths
        if !path.contains("/api/secured") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_boxed_body()) });
        }

        // Extract the JWT token from the Authorization header
        // Format: "Bearer <token>"
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

        // Create an authentication client to validate the token
        // Dereference Rc<String> to String for AuthClient::new
        let auth_client = AuthClient::new(
            format!("{}", auth_service_url),
            api_key.as_ref().to_string(), // Convert Rc<String> to String
        );

        let srv = Arc::clone(&self.service);

        Box::pin(async move {
            if let Some(token) = token_value {
                // Validate token and insert claims to request object for future usage
                match auth_client.validate_token(&token).await {
                    Ok(claims) => {
                        // Add the validated claims to the request extensions
                        // This makes the claims available to route handlers
                        req.extensions_mut().insert(claims);

                        // Continue processing the request with the wrapped service
                        srv.call(req).await.map(|res| res.map_into_boxed_body())
                    }
                    Err(_) => {
                        // Return 401 Unauthorized if the token is invalid
                        let response = HttpResponse::Unauthorized()
                            .json(serde_json::json!({"error": "Invalid token"}))
                            .map_into_boxed_body();
                        Ok(req.into_response(response))
                    }
                }
            } else {
                // Return 401 Unauthorized if no token is provided
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({"error": "No authorization token provided"}))
                    .map_into_boxed_body();
                Ok(req.into_response(response))
            }
        })
    }
}
