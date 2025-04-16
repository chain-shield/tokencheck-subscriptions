mod cors;

use actix_web::{web, App, HttpServer};
use common::env_config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // get env vars
    let config = Config::from_env();
    let config_data = config.clone();

    // get info
    let logger_enabled = config_data.console_logging_enabled;
    let is_production = config.environment == "production";
    let origin = config.cors_allowed_origin.clone();
    // let cookie_secure = !origin.contains("localhost");

    // init logger
    if logger_enabled {
        logger::setup().expect("Failed to set up logger");
    }

    // init db connection
    let pool = db::setup(&config.database_url, is_production)
        .await
        .expect("Failed to set up database");

    HttpServer::new(move || {
        App::new()
            .wrap(logger::middleware(logger_enabled))
            .wrap(cors::default(&origin))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config_data.clone()))
            .service(
                web::scope("/api")
                    .service(api_subs::mount_webhook())
                    .service(api_subs::mount_subs())
                    .service(
                        web::scope("/secured")
                            .wrap(api_auth::auth_middleware(config_data.clone()))
                            .service(api_subs::mount_pay())
                            .service(api_subs::mount_secure_subs()),
                    ),
            )
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .workers(config.num_workers)
    .run()
    .await
}
