use std::sync::Arc;

use actix_web::{post, web, Responder};
use serde::{Deserialize, Serialize};

use crate::common::{env_config::Config, error::Res, http::Success, stripe};

#[derive(Debug, Deserialize)]
pub struct CreateCustomerRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCustomerResponse {
    pub customer_id: String,
}

/// Create a new Stripe customer
///
/// This endpoint is intended to be called by other microservices (especially authentication)
/// when creating a new user.
#[post("/create-customer")]
pub async fn create_customer(
    config: web::Data<Arc<Config>>,
    user_data: web::Json<CreateCustomerRequest>,
) -> Res<impl Responder> {
    // Create Stripe customer
    let name = format!("{} {}", user_data.first_name, user_data.last_name);
    log::info!("Connecting to stripe client with stripe secret key...");
    let client = stripe::create_client(&config.stripe_secret_key);
    log::info!("Creating stripe customer for {}", name);
    let stripe_customer = stripe::create_customer(&client, &user_data.email, &name).await?;

    // Return the customer ID
    let response = CreateCustomerResponse {
        customer_id: stripe_customer.id.to_string(),
    };

    Success::created(response)
}
