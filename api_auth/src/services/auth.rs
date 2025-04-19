use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use common::{
    env_config::Config,
    error::{AppError, Res},
};
use db::models::user::User;
use oauth2::basic::*;
use oauth2::*;
use sqlx::PgPool;

use crate::{
    dtos::auth::{LoginRequest, OAuthUserData},
    misc::oauth::OAuthProvider,
};

/// Create OAuth client object.
///
/// # Arguments
///
/// * `provider` - The OAuth provider.
/// * `config` - The application configuration.
///
/// # Returns
///
/// A `Client` object for the specified OAuth provider.
pub fn create_oauth_client(
    provider: &OAuthProvider,
    config: &Config,
) -> Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
> {
    let provider_client = match provider {
        OAuthProvider::GitHub => &config.github_client,
        OAuthProvider::Google => &config.google_client,
        OAuthProvider::Facebook => &config.facebook_client,
        OAuthProvider::Apple => &config.apple_client,
        OAuthProvider::X => &config.x_client,
        // _ => panic!("Unsupported OAuth provider"),
    };

    let client_id = ClientId::new(provider_client.client_id.clone());
    let client_secret = ClientSecret::new(provider_client.client_secret.clone());
    let auth_url =
        AuthUrl::new(provider_client.auth_url.clone()).expect("Invalid authorization endpoint URL");
    let token_url =
        TokenUrl::new(provider_client.token_url.clone()).expect("Invalid token endpoint URL");

    let client = BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(
            RedirectUrl::new(provider_client.redirect_uri.to_string())
                .expect("Invalid redirect URL"),
        );

    client
}

/// Authenticates existing user.
/// If user does not exists, returns 400
/// If password hash does not match stored password hash, returns 401
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `login_data` - The login data.
///
/// # Returns
///
/// A `Result` containing the `User` object or an `AppError` if an error occurs.
pub async fn authenticate_user(pool: &PgPool, login_data: &LoginRequest) -> Res<User> {
    let (user, credentials) = db::user::get_user_with_password_hash(pool, login_data.email.clone())
        .await
        .map_err(|_| AppError::BadRequest("User with this email does not exist".to_string()))?;

    let parsed_hash = PasswordHash::new(&credentials.password_hash).unwrap();
    let is_valid = Argon2::default()
        .verify_password(login_data.password.as_bytes(), &parsed_hash)
        .is_ok();

    if is_valid {
        Ok(user)
    } else {
        Err(AppError::Unauthorized("Invalid credentials".to_string()))
    }
}

/// Fetches additional user data from providers OAuth API.
///
/// # Arguments
///
/// * `provider` - The OAuth provider.
/// * `access_token` - The access token.
///
/// # Returns
///
/// A `Result` containing the `OAuthUserData` object or an `AppError` if an error occurs.
pub async fn fetch_provider_user_data(
    provider: &OAuthProvider,
    access_token: &str,
) -> Res<OAuthUserData> {
    match provider {
        OAuthProvider::GitHub => fetch_github_user_data(access_token).await,
        OAuthProvider::Google => fetch_google_user_data(access_token).await,
        OAuthProvider::Facebook => fetch_facebook_user_data(access_token).await,
        OAuthProvider::X => fetch_x_user_data(access_token).await,
        prov => Err(AppError::Internal(format!(
            "Unsupported OAuth provider: {:?}",
            prov
        ))),
    }
}

async fn fetch_github_user_data(access_token: &str) -> Res<OAuthUserData> {
    let client = reqwest::Client::new();
    let request = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "WebServer");

    let response = request
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch GitHub user data: {}", e)))?;

    if response.status().is_success() {
        let github_user: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse GitHub user data: {}", e)))?;

        let email = match github_user["email"].as_str() {
            Some(email) => email.to_string(),
            None => {
                // If the email is not public, fetch it from the emails API
                let emails_url = "https://api.github.com/user/emails";
                let emails_response = client
                    .get(emails_url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .header("User-Agent", "WebServer")
                    .send()
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("Failed to fetch GitHub emails: {}", e))
                    })?;

                if emails_response.status().is_success() {
                    let emails: Vec<serde_json::Value> =
                        emails_response.json().await.map_err(|e| {
                            AppError::Internal(format!("Failed to parse GitHub emails: {}", e))
                        })?;

                    // Find the primary email
                    emails
                        .iter()
                        .find(|email| email["primary"].as_bool().unwrap_or(false))
                        .and_then(|email| email["email"].as_str())
                        .unwrap_or("")
                        .to_string()
                } else {
                    log::warn!(
                        "Failed to fetch GitHub emails: {:?}",
                        emails_response.status()
                    );
                    "".to_string()
                }
            }
        };
        let name = github_user["name"].as_str().unwrap_or("").to_string();
        let names: Vec<&str> = name.split(' ').collect();
        let first_name = names.first().unwrap_or(&"").to_string();
        let last_name = names.get(1).unwrap_or(&"").to_string();
        let provider_user_id = github_user["id"].to_string();

        Ok(OAuthUserData {
            email,
            first_name,
            last_name,
            provider_user_id,
        })
    } else {
        Err(AppError::Internal(format!(
            "GitHub API returned error status: {}",
            response.status()
        )))
    }
}

async fn fetch_google_user_data(access_token: &str) -> Res<OAuthUserData> {
    let client = reqwest::Client::new();
    let request = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .header("Authorization", format!("Bearer {}", access_token));

    let response = request
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch Google user data: {}", e)))?;

    if response.status().is_success() {
        let google_user: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse Google user data: {}", e)))?;

        let email = google_user["email"].as_str().unwrap_or("").to_string();
        let first_name = google_user["given_name"].as_str().unwrap_or("").to_string();
        let last_name = google_user["family_name"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let provider_user_id = google_user["sub"].to_string();

        Ok(OAuthUserData {
            email,
            first_name,
            last_name,
            provider_user_id,
        })
    } else {
        Err(AppError::Internal(format!(
            "Google API returned error status: {}",
            response.status()
        )))
    }
}

async fn fetch_facebook_user_data(access_token: &str) -> Res<OAuthUserData> {
    let client = reqwest::Client::new();
    let request = client
        .get("https://graph.facebook.com/me")
        .query(&[("fields", "email,first_name,last_name")])
        .header("Authorization", format!("Bearer {}", access_token));

    let response = request
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch Facebook user data: {}", e)))?;

    if response.status().is_success() {
        let facebook_user: serde_json::Value = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse Facebook user data: {}", e))
        })?;

        let email = facebook_user["email"].as_str().unwrap_or("").to_string();
        let first_name = facebook_user["first_name"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let last_name = facebook_user["last_name"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let provider_user_id = facebook_user["id"].to_string();

        Ok(OAuthUserData {
            email,
            first_name,
            last_name,
            provider_user_id,
        })
    } else {
        Err(AppError::Internal(format!(
            "Facebook API returned error status: {}",
            response.status()
        )))
    }
}

async fn fetch_x_user_data(access_token: &str) -> Res<OAuthUserData> {
    let client = reqwest::Client::new();
    let request = client
        .get("https://api.x.com/2/users/me")
        .query(&[("include_email", "true")])
        .header("Authorization", format!("Bearer {}", access_token));

    let response = request
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch X user data: {}", e)))?;

    if response.status().is_success() {
        let x_user: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse X user data: {}", e)))?;
        let provider_user_id = x_user["id"].as_str().unwrap_or("").to_string();
        let email = x_user["email"].as_str().unwrap_or("").to_string();
        let name = x_user["name"].as_str().unwrap_or("").to_string();
        let parts: Vec<&str> = name.split(' ').collect();
        let first_name = parts.get(0).unwrap_or(&"").to_string();
        let last_name = parts.get(1..).unwrap_or(&[""]).join(" ");
        Ok(OAuthUserData {
            email,
            first_name,
            last_name,
            provider_user_id,
        })
    } else {
        Err(AppError::Internal(format!(
            "X API returned error status: {}",
            response.status()
        )))
    }
}
