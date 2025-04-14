use log::{debug, info};
mod cors;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::PgPool;

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
    let cookie_secure = !origin.contains("localhost");

    // init logger
    if logger_enabled {
        logger::setup().expect("Failed to set up logger");
    }

    // init db connection
    let pool = db::setup(&config.database_url, is_production)
        .await
        .expect("Failed to set up database");

    HttpServer::new(move || {
        let secret = config_data.jwt_config.secret.as_bytes();
        App::new()
            .wrap(logger::middleware(logger_enabled))
            .wrap(cors::default(&origin))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config_data.clone()))
            .wrap(api_auth::session_middleware(
                cookie_secure,
                is_production,
                secret,
            ))
            .service(
                web::scope("/api")
                    .service(api_auth::mount_auth())
                    .service(api_subs::mount_webhook())
                    .service(
                        web::scope("/secured")
                            .wrap(api_auth::auth_middleware(config_data.clone()))
                            .service(api_auth::mount_user())
                            .service(api_subs::mount_pay())
                            .service(api_subs::mount_subs()),
                    ),
            )
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .workers(config.num_workers)
    .run()
    .await
}

async fn setup_database(config: &Config) -> Result<PgPool, Box<dyn std::error::Error>> {
    // Extract database name using regex or other parsing methods
    // This is a simple example - you might need to adjust based on your URL format
    let url = url::Url::parse(&config.database_url)?;
    let db_name = url.path().trim_start_matches('/');

    // Get username and password from the URL
    let username = url.username();
    let password = url.password().unwrap_or("");

    // Get host and port
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(5432);

    debug!("creating connection string...");
    // Create a connection string to the postgres database
    let admin_url = format!(
        "postgresql://{}:{}@{}:{}/postgres",
        username, password, host, port
    );
    debug!("username => {}", username);
    debug!("password => {}", password);
    debug!("host => {}", host);
    debug!("port => {}", port);
    debug!("admin_url => {}", admin_url);

    let db_ssl_mode = match config.db_ssl_mode.to_lowercase().as_str() {
        "required" => PgSslMode::Require,
        "disable" => PgSslMode::Disable,
        _ => PgSslMode::Require,
    };
    debug!("db ssl mode => {:?}", db_ssl_mode);

    // Connect to the 'postgres' database
    debug!("conencting to postgres db...");
    let admin_options = admin_url.parse::<PgConnectOptions>()?.ssl_mode(db_ssl_mode);
    let admin_pool = PgPool::connect_with(admin_options).await?;

    // Check if the target database exists
    debug!("check db exists..");
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(db_name)
            .fetch_one(&admin_pool)
            .await?;

    // Create the database if it doesn't exist
    debug!("create db connection..");
    if !exists {
        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&admin_pool)
            .await?;
    }

    // Close the admin connection
    debug!("close db..");
    admin_pool.close().await;

    // Connect to the target database
    debug!("connect to main db...");
    let options = config
        .database_url
        .parse::<PgConnectOptions>()?
        .ssl_mode(db_ssl_mode);
    let pool = PgPool::connect_with(options).await?;

    Ok(pool)
}
