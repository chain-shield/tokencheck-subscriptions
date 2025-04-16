use actix_session::Session;
use actix_web::{HttpResponse, Responder, get, http::header::LOCATION, post, web};
use common::env_config::Config;
use common::error::{AppError, Res};
use common::http::Success;
use common::jwt::{self, ClaimsSpec};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse, reqwest};
use sqlx::PgPool;
use std::sync::Arc;

use crate::dtos::auth::{AuthResponse, LoginRequest, OAuthCallbackQuery, RegisterRequest};
use crate::misc::oauth::OAuthProvider;
use crate::services;

/// Registers a new user with email and password authentication.
///
/// # Input
/// - `req`: JSON payload containing registration information (email, password, names)
/// - `pool`: Database connection pool
/// - `config`: Application configuration
///
/// # Output
/// - Success: Returns the created user object with 201 Created status
/// - Error: Returns 400 Bad Request if the email already exists
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API
/// const response = await fetch('/api/auth/register', {
///   method: 'POST',
///   headers: {
///     'Content-Type': 'application/json'
///   },
///   body: JSON.stringify({
///     email: 'user@example.com',
///     password: 'securepassword',
///     first_name: 'John',
///     last_name: 'Doe',
///     company_name: 'ACME Inc' // Optional
///   })
/// });
///
/// if (response.ok) {
///   const userData = await response.json();
///   console.log('Registered user:', userData);
/// }
/// ```
#[post("/register")]
async fn post_register(
    req: web::Json<RegisterRequest>,
    pool: web::Data<Arc<sqlx::PgPool>>,
    config: web::Data<Arc<Config>>,
) -> impl Responder {
    let pg_pool: &PgPool = &**pool;
    let username_exists = services::user::exists_user_by_email(pg_pool, req.email.clone()).await?;
    if username_exists {
        return Err(AppError::BadRequest("Username already exists".to_string()));
    }
    let user =
        services::user::create_user_with_credentials(pg_pool, &req.into_inner(), &config).await?;
    Ok(Success::created(user))
}

/// Authenticates a user with email and password.
///
/// # Input
/// - `login_data`: JSON payload containing email and password
/// - `config`: Application configuration for JWT generation
/// - `pool`: Database connection pool
///
/// # Output
/// - Success: Returns an auth response with JWT token and user details
/// - Error: Returns 401 Unauthorized for invalid credentials
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API
/// const response = await fetch('/api/auth/login', {
///   method: 'POST',
///   headers: {
///     'Content-Type': 'application/json'
///   },
///   body: JSON.stringify({
///     email: 'user@example.com',
///     password: 'securepassword'
///   })
/// });
///
/// if (response.ok) {
///   const authData = await response.json();
///   // Store token for authenticated requests
///   localStorage.setItem('authToken', authData.token);
///   console.log('Logged in user:', authData.user);
/// }
/// ```
#[post("/login")]
pub async fn post_login(
    login_data: web::Json<LoginRequest>,
    config: web::Data<Arc<Config>>,
    pool: web::Data<Arc<PgPool>>,
) -> Res<impl Responder> {
    let pg_pool: &PgPool = &**pool;
    let user = services::auth::authenticate_user(pg_pool, &login_data.into_inner()).await?;
    let token = jwt::generate_jwt(
        ClaimsSpec {
            user_id: user.id.clone(),
            stripe_customer_id: user.stripe_customer_id.clone(),
        },
        &config.jwt_config,
    )?;
    Success::ok(AuthResponse { token, user })
}

/// Initiates OAuth authentication flow with the specified provider.
///
/// # Input
/// - `path`: OAuth provider name (google, github, facebook, x, apple)
/// - `config`: Application configuration with OAuth settings
///
/// # Output
/// - Success: Redirects user to the OAuth provider's authentication page
/// - Error: Returns 400 Bad Request for invalid provider names
///
/// # Frontend Example
/// ```javascript
/// // This is typically a redirect link in your frontend, not an API call
/// // Example in React:
///
/// function OAuthButton({ provider }) {
///   return (
///     <a href={`/api/auth/oauth/${provider}`} className="oauth-button">
///       Login with {provider}
///     </a>
///   );
/// }
///
/// // Usage: <OAuthButton provider="github" />
/// ```
#[get("oauth/{provider}")]
pub async fn get_auth_provider(
    path: web::Path<String>,
    config: web::Data<Arc<Config>>,
) -> Res<impl Responder> {
    let provider = OAuthProvider::from_str(path.as_str())?;
    let client = services::auth::create_oauth_client(&provider, &config);

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(
            provider
                .get_scopes()
                .into_iter()
                .map(|s| Scope::new(s.to_string())),
        )
        .url();

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish())
}

/// Handles OAuth callback after user authenticates with the provider.
///
/// # Input
/// - `path`: OAuth provider name (google, github, facebook, x, apple)
/// - `query`: Query parameters containing the authorization code from the OAuth provider
/// - `config`: Application configuration
/// - `pool`: Database connection pool
/// - `session`: User session for storing authentication data
///
/// # Output
/// - Success: Redirects to the application callback URL with session data set
/// - Error: Returns appropriate error responses for various failure scenarios
///
/// # Note
/// This endpoint is not called directly from your frontend code.
/// It's the redirect URL configured with your OAuth provider that users
/// are sent to after authenticating with the provider.
///
/// The frontend application should have a route that matches the
/// configured web_app_auth_callback_url to handle the redirect after
/// successful authentication.
#[get("oauth/{provider}/callback")]
async fn get_auth_provider_callback(
    path: web::Path<String>,
    query: web::Query<OAuthCallbackQuery>,
    config: web::Data<Arc<Config>>,
    pool: web::Data<Arc<PgPool>>,
    session: Session,
) -> Res<impl Responder> {
    let provider = OAuthProvider::from_str(path.as_str())
        .map_err(|_| AppError::BadRequest("Invalid provider".to_string()))?;
    let client = services::auth::create_oauth_client(&provider, &config);
    let pg_pool: &PgPool = &**pool;

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token = client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(&http_client)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to exchange code. {}", e)))?;

    let access_token = token.access_token().secret();
    let user_data = services::auth::fetch_provider_user_data(&provider, access_token).await?;

    let existing_user =
        services::user::exists_user_by_email(pg_pool, user_data.email.clone()).await?;

    let auth_response = if existing_user {
        let user = services::user::get_user_by_email(pg_pool, user_data.email).await?;
        let token = jwt::generate_jwt(
            ClaimsSpec {
                user_id: user.id.clone(),
                stripe_customer_id: user.stripe_customer_id.clone(),
            },
            &config.jwt_config,
        )?;
        AuthResponse { token, user }
    } else {
        let user =
            services::user::create_user_with_oauth(pg_pool, &user_data, &provider, &config).await?;
        let token = jwt::generate_jwt(
            ClaimsSpec {
                user_id: user.id.clone(),
                stripe_customer_id: user.stripe_customer_id.clone(),
            },
            &config.jwt_config,
        )?;
        AuthResponse { token, user }
    };

    let user_string = serde_json::to_string(&auth_response.user).unwrap();
    let redirect_uri = config.web_app_auth_callback_url.as_str();

    session
        .insert("token", &auth_response.token)
        .map_err(|_| AppError::Internal("Failed to insert token cookie".to_string()))?;
    session
        .insert("user", &user_string)
        .map_err(|_| AppError::Internal("Failed to insert user cookie".to_string()))?;

    Ok(HttpResponse::Found()
        .append_header((LOCATION, redirect_uri))
        .finish())
}
