use std::sync::Arc;

use common::env_config::Config;
use middleware::auth::AuthMiddleware;

pub mod middleware {
    pub mod auth;
}
mod services {
    pub(crate) mod auth_client;
}

// Auth middleware
pub fn auth_middleware(config: Arc<Config>) -> AuthMiddleware {
    AuthMiddleware::new(config.auth_service_url.clone(), config.auth_api_key.clone())
}
