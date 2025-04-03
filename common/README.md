# Common module

This module provides functionalities that are common across all other modules.

## Configuration (`Config`, `JwtConfig`, `OAuthProviderClient`)

The application's configuration is managed through the `Config` struct, which encapsulates all necessary settings for the server to operate correctly. This includes database connections, JWT authentication, server parameters, OAuth provider client configurations, and Stripe API keys.

**Key Components:**

* **`Config`:**
    * Holds the main server configuration, loaded from environment variables.
    * Includes settings for database URL, JWT, server host and port, worker threads, CORS, logging, OAuth provider clients, and Stripe API keys.
    * Provides a `from_env()` method to initialize the configuration from environment variables with sensible defaults.
* **`JwtConfig`:**
    * Manages JWT authentication settings, including the secret key and token expiration time.
    * Initializes from environment variables `JWT_SECRET` and `JWT_EXPIRATION_HOURS`.
* **`OAuthProviderClient`:**
    * Stores configuration for OAuth 2.0 providers (GitHub, Google, Facebook, Apple, X).
    * Includes client ID, client secret, authentication and token URLs, and redirect URI.
* **Environment Variable Loading:**
    * The configuration is primarily loaded from environment variables, allowing for flexible deployment and easy updates without code changes.
    * `dotenvy` crate is used to load .env files in development environments.
* **Default Values:**
    * Sensible default values are provided for optional configuration parameters, simplifying setup.
* **Error Handling:**
    * The `from_env()` methods panic if required environment variables are missing or if numeric values cannot be parsed correctly, ensuring early detection of configuration issues.
* **Stripe Configuration:**
    * Includes `stripe_secret_key` and `stripe_webhook_secret` to configure the Stripe API.

**Usage:**

The `Config::from_env()` method is used to create an `Arc<Config>` instance, which is then used throughout the application to access configuration settings. The `JwtConfig::from_env()` method is used to create a `JwtConfig` struct, and the `OAuthProviderClient` struct is populated in the `Config::from_env()` method.

## Error Handling (`AppError`)

The application utilizes a custom error enum, `AppError`, for centralized and consistent error handling. This enum encapsulates various error types, including database, JWT, HTTP request, Stripe, and application-specific errors.

**Key Features:**

* **Comprehensive Error Types:** Covers a wide range of potential errors, ensuring detailed error reporting.
* **`thiserror` Integration:** Simplifies error definition and provides human-readable error messages.
* **`actix_web::ResponseError` Implementation:** Enables seamless integration with Actix Web, automatically converting `AppError` instances into HTTP responses.
* **HTTP Response Mapping:** Each `AppError` variant is mapped to an appropriate HTTP status code (e.g., 400 Bad Request, 401 Unauthorized, 500 Internal Server Error).
* **Logging:** Internal server errors are logged for debugging purposes.

**Usage:**

The `Res<T>` type alias is defined as `std::result::Result<T, AppError>`, making it easy to return `AppError` instances from functions. When an `AppError` is returned, Actix Web automatically generates an HTTP response based on the error type.

### Other
- HTTP response helper functions
- Stripe helper functions
- JWT claims