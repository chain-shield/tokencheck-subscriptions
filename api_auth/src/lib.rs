use std::sync::Arc;

use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{
    cookie::{Key, SameSite, time::Duration},
    web,
};
use common::env_config::Config;
use middleware::auth::AuthMiddleware;

pub mod routes {
    pub mod auth;
    pub mod session;
    pub mod user;
}
pub mod middleware {
    pub mod auth;
}
mod services {
    pub(crate) mod auth;
    pub(crate) mod user;
}
mod misc {
    pub(crate) mod oauth;
}
mod dtos {
    pub(crate) mod auth;
}

// Auth middleware
pub fn auth_middleware(config: Arc<Config>) -> AuthMiddleware {
    AuthMiddleware::new(config.jwt_config.clone())
}
// Session middleware
pub fn session_middleware(
    cookie_secure: bool,
    is_production: bool,
    secret: &[u8],
) -> SessionMiddleware<CookieSessionStore> {
    let secret_key = Key::derive_from(secret);
    SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
        .cookie_name("auth_session".to_string())
        .cookie_secure(cookie_secure)
        .cookie_same_site(if cookie_secure {
            SameSite::None
        } else {
            SameSite::Lax
        })
        .cookie_http_only(true)
        .cookie_domain(if is_production {
            Some(".somedomain.com".to_string()) // TODO: change domain
        } else {
            None
        })
        .session_lifecycle(PersistentSession::default().session_ttl(Duration::hours(24)))
        .build()
}
// Auth endpoints
pub fn mount_auth() -> actix_web::Scope {
    web::scope("/auth")
        .service(routes::session::get_session)
        .service(routes::auth::post_register)
        .service(routes::auth::post_login)
        .service(routes::auth::get_auth_provider)
        .service(routes::auth::get_auth_provider_callback)
}
// User endpoints
pub fn mount_user() -> actix_web::Scope {
    web::scope("/user").service(routes::user::get_me)
}
