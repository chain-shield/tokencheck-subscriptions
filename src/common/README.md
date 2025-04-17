# Common Module

This module provides shared utilities and functionality used throughout the application.

## Module Structure

```
common/
├── env_config.rs # Environment configuration
├── error.rs      # Error handling
├── http.rs       # HTTP utilities
├── jwt.rs        # JWT handling
├── misc.rs       # Miscellaneous utilities
└── stripe.rs     # Stripe integration utilities
```

## Features

### Environment Configuration

The `env_config.rs` file defines the `Config` struct that loads and provides access to environment variables:

```rust
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub num_workers: usize,
    pub database_url: String,
    pub environment: String,
    pub cors_allowed_origin: String,
    pub console_logging_enabled: bool,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub auth_service_url: String,
    pub auth_api_key: String,
}
```

Key methods:
- `from_env() -> Self`: Loads configuration from environment variables

### Error Handling

The `error.rs` file defines the `AppError` enum for consistent error handling throughout the application:

```rust
pub enum AppError {
    Internal(String),
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    UnprocessableEntity(String),
    ServiceUnavailable(String),
}
```

It also provides:
- Conversions from common error types (sqlx::Error, serde_json::Error, etc.)
- Implementation of `ResponseError` for integration with Actix Web
- The `Res<T>` type alias for `Result<T, AppError>`

### HTTP Utilities

The `http.rs` file provides the `Success` struct for consistent success responses:

```rust
pub struct Success<T: Serialize> {
    pub data: T,
}
```

With methods for different HTTP status codes:
- `ok(data: T) -> HttpResponse`
- `created(data: T) -> HttpResponse`
- `accepted(data: T) -> HttpResponse`
- `no_content() -> HttpResponse`

### JWT Handling

The `jwt.rs` file defines the `Claims` struct for JWT token claims:

```rust
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub user_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub stripe_customer_id: String,
    pub plan_id: Uuid,
    pub sub_status: String,
}
```

### Miscellaneous Utilities

The `misc.rs` file provides various utility functions and types, including:

```rust
pub enum UserVerificationOrigin {
    Email,
    OAuth,
}
```

### Stripe Integration

The `stripe.rs` file provides utilities for Stripe integration:

```rust
pub fn create_client(secret_key: &str) -> Client
```

## Usage

The common module is used throughout the application:

- `Config` is used in `main.rs` to load environment variables
- `AppError` is used for error handling in all modules
- `Success` is used for HTTP responses in API endpoints
- `Claims` is used for JWT token handling in the auth module
- Stripe utilities are used in the api_subs module
