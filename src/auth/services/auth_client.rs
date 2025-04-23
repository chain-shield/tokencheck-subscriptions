// auth_client.rs (in the calling microservice)
use log::{info, warn};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenValidationRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenValidationResponse {
    pub user_id: Uuid,
    pub stripe_customer_id: String,
    pub exp: u32,
}

pub struct AuthClient {
    client: Client,
    auth_service_url: String,
    api_key: String,
}

impl AuthClient {
    pub fn new(auth_service_url: String, api_key: String) -> Self {
        AuthClient {
            client: Client::new(),
            auth_service_url,
            api_key,
        }
    }

    pub async fn validate_token(&self, token: &str) -> anyhow::Result<TokenValidationResponse> {
        let request_body = TokenValidationRequest {
            token: token.to_string(),
        };

        info!(
            "Sending token validation request to {}",
            format!("{}/validate/validate-token", self.auth_service_url)
        );
        let response = self
            .client
            .post(format!("{}/validate/validate-token", self.auth_service_url))
            .json(&request_body)
            .header("X-API-Key", &self.api_key) // Include the API key in the header
            .send()
            .await?;

        log::info!("← auth service answered: {}", response.status());

        if !response.status().is_success() {
            let error_response = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or(serde_json::json!({"error": "Unknown error", "message": "Failed to validate token"}));
            let message = error_response["message"]
                .as_str()
                .unwrap_or("Failed to validate token")
                .to_string();
            warn!("Token validation failed: {}", message);
            return Err(anyhow::anyhow!(message));
        }

        log::info!("token validation sucessfull..");
        let token_response = response.json::<TokenValidationResponse>().await?;
        info!("... for user_id: {}", token_response.user_id);
        Ok(token_response)
    }
}
