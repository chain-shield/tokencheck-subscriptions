use actix_session::Session;
use actix_web::{Responder, get, web};
use common::error::{AppError, Res};
use db::models::user::User;
use serde_json::json;

/// Retrieves current session data for the authenticated user from session cookies.
///
/// # Input
/// - `session`: The user's session containing authentication data
///
/// # Output
/// - Success: Returns JSON with user data and token
/// - Error: Returns 401 Unauthorized if no valid session exists
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API with credentials to include cookies
/// const response = await fetch('/session', {
///   credentials: 'include' // Important for sending session cookies
/// });
///
/// if (response.ok) {
///   const sessionData = await response.json();
///   console.log('Session token:', sessionData.token);
///   console.log('Session user:', sessionData.user);
///   
///   // Store the token for API calls
///   localStorage.setItem('authToken', sessionData.token);
/// } else if (response.status === 401) {
///   console.log('No active session found, user needs to login');
///   // Redirect to login page
///   window.location.href = '/login';
/// }
/// ```
#[get("/session")]
async fn get_session(session: Session) -> Res<impl Responder> {
    let user = session
        .get::<String>("user")
        .map_err(|_| AppError::BadRequest("Session user error".to_string()))?
        .ok_or_else(|| AppError::Unauthorized("No user data found".to_string()))?;
    let token = session
        .get::<String>("token")
        .map_err(|_| AppError::BadRequest("Session token error".to_string()))?
        .ok_or_else(|| AppError::Unauthorized("No session token found".to_string()))?;

    Ok(web::Json(json!({
        "token": token,
        "user": serde_json::from_str::<User>(&user).map_err(|_| AppError::Internal("Failed to parse user json".to_string()))?
    })))
}
