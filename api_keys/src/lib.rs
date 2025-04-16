use actix_web::web;
use middleware::key::KeyMiddleware;

pub mod routes {
    pub mod key;
    pub mod usage;
}
pub mod middleware {
    pub mod key;
}

mod service {
    pub(crate) mod key;
    pub(crate) mod usage;
}
mod dtos {
    pub(crate) mod key;
    pub(crate) mod usage;
}

pub fn mount_keys() -> actix_web::Scope {
    web::scope("/key")
        .service(routes::key::get_keys)
        .service(routes::key::post_generate_key)
        .service(routes::key::post_revoke)
        .service(routes::usage::get_usage)
}
pub fn middleware() -> KeyMiddleware {
    KeyMiddleware::new()
}
