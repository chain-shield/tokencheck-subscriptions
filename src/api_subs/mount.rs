use actix_web::web::{self};
use crate::api_subs::routes;

pub fn mount_secure_subs() -> actix_web::Scope {
    web::scope("/sub")
        .service(routes::sub::post_subscribe)
        .service(routes::sub::get_current)
        .service(routes::sub::post_auto_renew)
}
pub fn mount_subs() -> actix_web::Scope {
    web::scope("/sub").service(routes::sub::get_plans)
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
