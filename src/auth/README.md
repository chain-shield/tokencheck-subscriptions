# Authentication Module

This module provides authentication functionality for the application, including middleware for securing API endpoints and services for interacting with the authentication service.

## Module Structure

```
auth/
├── middleware/       # Authentication middleware
│   └── auth.rs       # AuthMiddleware implementation
└── services/         # Authentication services
    └── auth_client.rs # Client for the authentication service
```

## Features

### Authentication Middleware

The `AuthMiddleware` in `middleware/auth.rs` is an Actix Web middleware that:

1. Extracts the JWT token from the Authorization header
2. Validates the token with the authentication service
3. Adds the validated claims to the request extensions
4. Rejects requests with invalid or missing tokens

Usage example:
```rust
.service(
    web::scope("/secured")
        .wrap(auth::AuthMiddleware::new(
            config_data.auth_service_url.clone(),
            config_data.auth_api_key.clone()
        ))
        .service(/* secured endpoints */)
)
```

### Authentication Client

The `AuthClient` in `services/auth_client.rs` provides a client for interacting with the authentication service:

```rust
pub struct AuthClient {
    client: Client,
    auth_service_url: String,
    api_key: String,
}
```

Key methods:
- `new(auth_service_url: String, api_key: String) -> Self`: Creates a new client
- `validate_token(&self, token: &str) -> anyhow::Result<TokenValidationResponse>`: Validates a JWT token

The `TokenValidationResponse` includes:
- `user_id`: The ID of the authenticated user
- `exp`: The token expiration timestamp
- `plan_id`: The user's subscription plan ID
- `sub_status`: The subscription status

## Integration with Main Application

The auth module is integrated with the main application in `main.rs`:

1. The `AuthMiddleware` is applied to secured API endpoints
2. The middleware uses the authentication service URL and API key from the application configuration

This ensures that only authenticated users with valid tokens can access the secured endpoints.
