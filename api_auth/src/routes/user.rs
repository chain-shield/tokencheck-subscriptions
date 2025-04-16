use std::sync::Arc;

use actix_web::{Responder, get, web};
use common::{http::Success, jwt::JwtClaims};
use sqlx::PgPool;

use crate::services;

/// Endpoint to retrieve the current authenticated user's information.
///
/// This handler extracts the user ID from the authentication claims and fetches
/// the corresponding user record from the database.
///
/// # Input
/// - `claims`: The JWT claims extracted from the authentication token, containing the user ID
/// - `pool`: A database connection pool for retrieving user data
///
/// # Output
/// - Success: Returns a JSON object with the user's profile information
/// - Error: Returns 401 Unauthorized if no valid token is provided or 404 Not Found if user doesn't exist
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API with the JWT token from login/registration
/// const response = await fetch('/api/secured/me', {
///   headers: {
///     'Authorization': `Bearer ${localStorage.getItem('authToken')}`
///   }
/// });
///
/// if (response.ok) {
///   const user = await response.json();
///   console.log('Current user:', user);
///   // Example user data:
///   // {
///   //   id: "a1b2c3d4-...",
///   //   email: "user@example.com",
///   //   first_name: "John",
///   //   last_name: "Doe",
///   //   company_name: "ACME Inc",
///   //   verified: true,
///   //   created_at: "2023-01-01T12:00:00Z",
///   //   ...
///   // }
/// }
/// ```
#[get("/me")]
async fn get_me(
    claims: web::ReqData<JwtClaims>,
    pool: web::Data<Arc<sqlx::PgPool>>,
) -> impl Responder {
    let user_id = claims.user_id;
    let pg_pool: &PgPool = &**pool;
    let user = services::user::get_user_by_id(pg_pool, user_id).await?;
    Success::ok(user)
}
