use actix_cors::Cors;
use actix_web::http::header::{self, HeaderName};

pub fn middleware(origin: &str) -> Cors {
    Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::COOKIE,
            header::SET_COOKIE,
            HeaderName::from_static("x-api-key"),
        ])
        .allowed_origin(origin)
        .expose_headers(&[header::SET_COOKIE])
        .supports_credentials()
        .max_age(3600)
}
