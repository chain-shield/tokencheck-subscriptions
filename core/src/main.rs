mod cors;

use actix_web::{
    App, HttpServer,
    web::{self},
};
use common::env_config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // get env vars
    let config = Config::from_env();
    let config_data = config.clone();

    // get info
    let is_production = config.environment == "production";
    let origin = config.cors_allowed_origin.clone();
    let cookie_secure = !origin.contains("localhost");

    // init logger
    if config.console_logging_enabled {
        logger::setup().expect("Failed to set up logger");
    }

    // init db connection
    let pool = db::setup(&config.database_url, is_production)
        .await
        .expect("Failed to set up database");

    // get all subscription plans from Stripe
    let client = common::stripe::create_client(&config.stripe_secret_key);
    let plans = api_subs::services::sub::get_subscription_plans(&client)
        .await
        .expect("Failed to fetch subscription plans from Stripe API");

    // init Redis
    let redis_client =
        redis::Client::open(config.redis_url.clone()).expect("Failed to create Redis client");

    HttpServer::new(move || {
        let secret = config_data.jwt_config.secret.as_bytes();
        let redis_client = redis_client.clone();
        let plans_data = plans.clone();
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config_data.clone()))
            .wrap(limiter::global_middleware(10)) // max 10 requests per second
            .wrap(logger::middleware()) // 4th
            .wrap(extractor::middleware()) // 3rd
            .wrap(cors::middleware(&origin)) // 2nd
            .wrap(api_auth::session_middleware(
                cookie_secure,
                is_production,
                secret,
            )) // 1st
            .service(
                web::scope("/api")
                    .service(api_auth::mount_auth())
                    .service(api_subs::mount_webhook())
                    .service(
                        web::scope("/dashboard")
                            .wrap(api_auth::auth_middleware())
                            .service(api_auth::mount_user())
                            .service(api_subs::mount_pay())
                            .service(api_subs::mount_subs())
                            .service(api_keys::mount_keys()),
                    )
                    .service(
                        web::scope("/v1")
                            .wrap(api_keys::middleware())
                            .wrap(limiter::quota_middleware(plans_data, redis_client))
                            .service(checker::mount_checker()),
                    ),
            )
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .workers(config.num_workers)
    .run()
    .await
}
