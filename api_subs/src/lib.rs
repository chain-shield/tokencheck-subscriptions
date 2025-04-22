use actix_web::web::{self};
use common::env_config::Config;
use redis::AsyncCommands;

pub mod routes {
    pub mod pay;
    pub mod sub;
}

pub mod services {
    pub mod pay;
    pub mod sub;
}

mod dtos {
    pub(crate) mod pay;
    pub(crate) mod sub;
}

pub mod models {
    pub mod sub;
}

mod misc {
    pub(crate) mod pay;
}

pub async fn setup(config: &Config, redis_pool: deadpool_redis::Pool) {
    let client = common::stripe::create_client(&config.stripe_secret_key);

    // fetch plans from stripe
    let plans = services::sub::get_subscription_plans(&client)
        .await
        .expect("Failed to fetch subscription plans from Stripe API");

    // write limits into redis
    let mut conn = redis_pool
        .get()
        .await
        .expect("Failed to get connection to Redis");
    for plan in plans {
        if let Some(meta) = &plan.metadata {
            let key = format!("plan:{}:limits", plan.id);
            let _: () = conn
                .hset_multiple(
                    &key,
                    &[
                        ("daily_api_limit", meta.daily_api_limit.as_str()),
                        ("monthly_api_limit", meta.monthly_api_limit.as_str()),
                    ],
                )
                .await
                .expect("Failed to write plan limits to Redis");
        }
    }
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
