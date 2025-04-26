use std::{env, sync::Arc};

#[derive(Clone, Debug)]
/// Configuration struct for the server.
///
/// This struct holds all the necessary configuration parameters
/// required to initialize and run the server.
/// It includes database connection details, JWT configuration,
/// server host and port, number of worker threads, CORS settings,
/// logging preferences, web application authentication callback URL,
/// and GitHub client configuration.
pub struct Config {
    // server to server api keys, other micro services MUST
    // use one of these keys to make call to this service
    pub subs_service_api_keys: Vec<String>,
    // url to reach authentcation service
    pub auth_service_url: String,
    // api key REQUIRED to make validate token call to auth service
    pub auth_api_key: String,
    // SSL mode , do we need secure connection to db?
    pub db_ssl_mode: String,
    // environment
    // jwt secret to decode jwt tokens
    pub jwt_secret: String,
    pub environment: String, // development or production
    /// The URL of the database to connect to.
    pub database_url: String,
    /// The hostname or IP address the server will bind to.
    pub server_host: String,
    /// The port number the server will listen on.
    pub server_port: u16,
    /// The number of worker threads to spawn for handling requests.
    pub num_workers: usize,
    /// The allowed origin for CORS (Cross-Origin Resource Sharing).
    pub cors_allowed_origin: String,
    /// A boolean indicating whether console logging is enabled.
    pub console_logging_enabled: bool,
    pub stripe_secret_key: String,
    /// Stripe webhook secret
    pub stripe_webhook_secret: String,
}

impl Config {
    /// Creates a new `Config` instance from environment variables.
    ///
    /// Loads all configuration values from environment variables with sensible defaults
    /// for most optional settings. This method initializes the complete server configuration
    /// including database connection, JWT settings, server parameters, and OAuth provider clients.
    ///
    /// # Environment Variables
    ///
    /// Required:
    /// - `DATABASE_URL`: Connection string for the database
    /// - `JWT_SECRET`: Secret key for JWT signing (via `JwtConfig::from_env()`)
    ///
    /// Optional (with defaults):
    /// - `IP`: Server host (default: "127.0.0.1")
    /// - `PORT`: Server port (default: 8080)
    /// - `WORKERS`: Number of worker threads (default: 4)
    /// - `CORS_ALLOWED_ORIGIN`: Allowed CORS origin (default: "http://localhost:3000")
    /// - `ENABLE_CONSOLE_LOGGING`: Whether to enable console logging (default: true)
    /// - `WEB_APP_AUTH_CALLBACK_URL`: Web app callback URL (default: "http://localhost:3000/auth/callback")
    /// - Various OAuth provider settings (see implementation for details)
    ///
    /// # Panics
    ///
    /// This function will panic if required environment variables are missing or if
    /// numeric values cannot be parsed correctly.

    pub fn from_env() -> Arc<Self> {
        dotenvy::dotenv().ok();

        let stripe_secret_key = env::var("STRIPE_SECRET_KEY").unwrap_or_default();
        let stripe_webhook_secret = env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default();

        Arc::new(Config {
            subs_service_api_keys: env::var("SUBS_SERVICE_API_KEYS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            auth_service_url: env::var("AUTH_SERVICE_URL").expect("AUTH_SERVICE_URL must be set"),
            auth_api_key: env::var("AUTH_API_KEY").expect("AUTH_API_KEY must be set"),
            db_ssl_mode: env::var("DB_SSL_MODE").expect("DB_SSL_MODE must be set"),
            environment: env::var("ENVIRONMENT").expect("ENVIRONMENT must be set"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            server_host: env::var("IP").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            num_workers: env::var("WORKERS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
            cors_allowed_origin: env::var("CORS_ALLOWED_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            console_logging_enabled: env::var("ENABLE_CONSOLE_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .to_lowercase()
                == "true",
            stripe_secret_key,
            stripe_webhook_secret,
        })
    }
}
