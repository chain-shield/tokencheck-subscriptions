mod cors;

use actix_web::{web, App, HttpServer};
use std::str::FromStr;
use tokencheck_subscriptions::{common::env_config::Config, api_subs, auth, logger};

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
    let url = url::Url::parse(&config.database_url).expect("Failed to parse database URL");
    let db_name = url.path().trim_start_matches('/');
    let username = url.username();
    let password = url.password().unwrap_or("");
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(5432);

    let admin_url = format!(
        "postgresql://{}:{}@{}:{}/postgres",
        username, password, host, port
    );

    let mut admin_options = sqlx::postgres::PgConnectOptions::from_str(&admin_url)
        .expect("Failed to create admin options");
    if is_production {
        admin_options = admin_options.ssl_mode(sqlx::postgres::PgSslMode::Require);
    }

    let admin_pool = sqlx::PgPool::connect_with(admin_options)
        .await
        .expect("Failed to connect to admin database");

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(db_name)
            .fetch_one(&admin_pool)
            .await
            .expect("Failed to check if database exists");

    if !exists {
        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&admin_pool)
            .await
            .expect("Failed to create database");
    }

    admin_pool.close().await;

    let mut options = sqlx::postgres::PgConnectOptions::from_str(&config.database_url)
        .expect("Failed to create options");
    if is_production {
        options = options.ssl_mode(sqlx::postgres::PgSslMode::Require);
    }
    let pool = std::sync::Arc::new(
        sqlx::PgPool::connect_with(options)
            .await
            .expect("Failed to connect to database")
    );

    HttpServer::new(move || {
        App::new()
            .wrap(logger::middleware(logger_enabled))
            .wrap(cors::default(&origin))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config_data.clone()))
            .service(
                web::scope("/api")
                    .service(api_subs::mount::mount_webhook())
                    .service(api_subs::mount::mount_subs())
                    .service(
                        web::scope("/secured")
                            .wrap(auth::AuthMiddleware::new(config_data.auth_service_url.clone(), config_data.auth_api_key.clone()))
                            .service(api_subs::mount::mount_pay())
                            .service(api_subs::mount::mount_secure_subs()),
                    ),
            )
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .workers(config.num_workers)
    .run()
    .await
}
