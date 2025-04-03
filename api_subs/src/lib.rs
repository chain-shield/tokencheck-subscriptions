use actix_web::web::{self};

pub mod routes {
    pub mod pay;
    pub mod sub;
}

mod services {
    pub(crate) mod pay;
    pub(crate) mod sub;
}

mod dtos {
    pub(crate) mod pay;
    pub(crate) mod sub;
}

mod models {
    pub(crate) mod sub;
}

mod misc {
    pub(crate) mod pay;
}

pub fn mount_subs() -> actix_web::Scope {
    web::scope("/sub")
        .service(routes::sub::get_plans)
        .service(routes::sub::post_subscribe)
        .service(routes::sub::get_current)
        .service(routes::sub::post_auto_renew)
}
pub fn mount_pay() -> actix_web::Scope {
    web::scope("/pay")
        .service(routes::pay::post_refund)
        .service(routes::pay::get_subscription_payment)
        .service(routes::pay::post_payment_intents)
}
pub fn mount_webhook() -> actix_web::Scope {
    web::scope("/pay").service(routes::pay::post_webhook)
}

