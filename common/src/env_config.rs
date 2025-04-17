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
    // environment
    pub environment: String, // development or production
    /// The URL of the database to connect to.
    pub database_url: String,
    /// The URL of Redis server to connect to.
    pub redis_url: String,
    /// Configuration for JWT (JSON Web Token) authentication.
    pub jwt_config: JwtConfig,
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
    /// The URL that the web application will redirect to after authentication.
    pub web_app_auth_callback_url: String,
    /// Configuration for the GitHub OAuth2 client.
    pub github_client: OAuthProviderClient,
    /// Configuration for the Google OAuth2 client.
    pub google_client: OAuthProviderClient,
    /// Configuration for the Facebook OAuth2 client.
    pub facebook_client: OAuthProviderClient,
    /// Configuration for the Apple OAuth2 client.
    pub apple_client: OAuthProviderClient,
    /// Configuration for the X OAuth2 client.
    pub x_client: OAuthProviderClient,
    /// Stripe secret key
    pub stripe_secret_key: String,
    /// Stripe webhook secret
    pub stripe_webhook_secret: String,
}

#[derive(Clone, Debug)]
/// `ProviderClient` holds the configuration necessary for interacting with an OAuth 2.0 provider.
///
/// It contains the client ID and secret, as well as the authentication and token URLs required
/// for the OAuth 2.0 flow. The redirect URI is also stored for use after successful authentication.
pub struct OAuthProviderClient {
    /// The client ID for the OAuth 2.0 provider.
    pub client_id: String,
    /// The client secret for the OAuth 2.0 provider.
    pub client_secret: String,
    /// The authentication URL for the OAuth 2.0 provider.
    pub auth_url: String,
    /// The token URL for the OAuth 2.0 provider.
    pub token_url: String,
    /// The redirect URI for the OAuth 2.0 provider.
    pub redirect_uri: String,
}

#[derive(Clone, Debug)]
/// Configuration for JSON Web Token (JWT) authentication.
///
/// This struct contains the secret key used to sign JWTs and
/// the expiration time in hours for issued tokens.
pub struct JwtConfig {
    /// The secret key used to sign and verify JWTs.
    pub secret: String,
    /// The expiration time for JWTs in hours.
    pub expiration_hours: i64,
}

impl JwtConfig {
    /// Creates a new `JwtConfig` instance from environment variables.
    ///
    /// Reads the JWT configuration from environment variables:
    /// - `JWT_SECRET`: Required. The secret key for JWT signing.
    /// - `JWT_EXPIRATION_HOURS`: Optional. Defaults to 24 hours if not provided.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - `JWT_SECRET` environment variable is not set
    /// - `JWT_EXPIRATION_HOURS` is set but cannot be parsed as a valid number
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        JwtConfig {
            secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRATION_HOURS must be a valid number"),
        }
    }
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
            environment: env::var("ENVIRONMENT").expect("ENVIRONMENT must be set"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set"),
            jwt_config: JwtConfig::from_env(),
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
            web_app_auth_callback_url: env::var("WEB_APP_AUTH_CALLBACK_URL")
                .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string()),
            github_client: OAuthProviderClient {
                client_id: env::var("GITHUB_CLIENT_ID").unwrap_or_default(),
                client_secret: env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
                auth_url: env::var("GITHUB_AUTH_URL")
                    .unwrap_or_else(|_| "https://github.com/login/oauth/authorize".to_string()),
                token_url: env::var("GITHUB_TOKEN_URL")
                    .unwrap_or_else(|_| "https://github.com/login/oauth/access_token".to_string()),
                redirect_uri: env::var("GITHUB_REDIRECT_URI").unwrap_or_else(|_| {
                    "http://localhost:8080/api/auth/oauth/github/callback".to_string()
                }),
            },
            google_client: OAuthProviderClient {
                client_id: env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
                client_secret: env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
                auth_url: env::var("GOOGLE_AUTH_URL")
                    .unwrap_or_else(|_| "https://accounts.google.com/o/oauth2/v2/auth".to_string()),
                token_url: env::var("GOOGLE_TOKEN_URL")
                    .unwrap_or_else(|_| "https://www.googleapis.com/oauth2/v4/token".to_string()),
                redirect_uri: env::var("GOOGLE_REDIRECT_URI").unwrap_or_else(|_| {
                    "http://localhost:8080/api/auth/oauth/google/callback".to_string()
                }),
            },
            facebook_client: OAuthProviderClient {
                client_id: env::var("FACEBOOK_CLIENT_ID").unwrap_or_default(),
                client_secret: env::var("FACEBOOK_CLIENT_SECRET").unwrap_or_default(),
                auth_url: env::var("FACEBOOK_AUTH_URL")
                    .unwrap_or_else(|_| "https://www.facebook.com/v9.0/dialog/oauth".to_string()),
                token_url: env::var("FACEBOOK_TOKEN_URL").unwrap_or_else(|_| {
                    "https://graph.facebook.com/v9.0/oauth/access_token".to_string()
                }),
                redirect_uri: env::var("FACEBOOK_REDIRECT_URI").unwrap_or_else(|_| {
                    "http://localhost:8080/api/auth/oauth/facebook/callback".to_string()
                }),
            },
            apple_client: OAuthProviderClient {
                client_id: env::var("APPLE_CLIENT_ID").unwrap_or_default(),
                client_secret: env::var("APPLE_CLIENT_SECRET").unwrap_or_default(),
                auth_url: env::var("APPLE_AUTH_URL")
                    .unwrap_or_else(|_| "https://appleid.apple.com/auth/authorize".to_string()),
                token_url: env::var("APPLE_TOKEN_URL")
                    .unwrap_or_else(|_| "https://appleid.apple.com/auth/token".to_string()),
                redirect_uri: env::var("APPLE_REDIRECT_URI").unwrap_or_else(|_| {
                    "http://localhost:8080/api/auth/oauth/apple/callback".to_string()
                }),
            },
            x_client: OAuthProviderClient {
                client_id: env::var("X_CLIENT_ID").unwrap_or_default(),
                client_secret: env::var("X_CLIENT_SECRET").unwrap_or_default(),
                auth_url: env::var("X_AUTH_URL")
                    .unwrap_or_else(|_| "https://x.com/i/oAuth2/authorize".to_string()),
                token_url: env::var("X_TOKEN_URL")
                    .unwrap_or_else(|_| "https://api.x.com/2/oAuth2/token".to_string()),
                redirect_uri: env::var("X_REDIRECT_URI").unwrap_or_else(|_| {
                    "http://localhost:8080/api/auth/oauth/x/callback".to_string()
                }),
            },
            stripe_secret_key,
            stripe_webhook_secret,
        })
    }
}
