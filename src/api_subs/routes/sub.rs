use actix_web::{Responder, get, post, web};
use crate::common::{env_config::Config, error::AppError, http::Success, jwt::Claims, stripe};
use std::sync::Arc;

use crate::api_subs::{
    dtos::{
        pay::SubscriptionRequest,
        sub::{
            EnterpriseSubscriptionRequest, SubscriptionCreateRequest, SubscriptionPlansResponse,
            SubscriptionResponse, UpdateAutoRenewRequest, UserSubscriptionResponse,
        },
    },
    services,
};

/// Retrieves all available subscription plans from Stripe.
///
/// # Input
/// - `config`: Application configuration with Stripe API credentials
///
/// # Output
/// - Success: Returns a JSON object containing an array of subscription plans
/// - Error: Returns 500 Internal Server Error if plans cannot be retrieved
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API with authorization
/// const response = await fetch('/api/secured/sub/plans', {
///   headers: {
///     'Authorization': `Bearer ${localStorage.getItem('authToken')}`
///   }
/// });
///
/// if (response.ok) {
///   const data = await response.json();
///   console.log('Available plans:', data.plans);
///   // Example response:
///   // {
///   //   plans: [
///   //     {
///   //       id: "price_123abc",
///   //       name: "Basic Plan",
///   //       description: "For small teams",
///   //       price: 1999, // in cents
///   //       currency: "usd",
///   //       interval: "month",
///   //       active: true,
///   //       features: ["Feature 1", "Feature 2"]
///   //     },
///   //     // More plans...
///   //   ]
///   // }
/// }
/// ```
#[get("/plans")]
pub async fn get_plans(config: web::Data<Arc<Config>>) -> impl Responder {
    let client = stripe::create_client(&config.stripe_secret_key);
    let plans = services::sub::get_subscription_plans(&client).await?;
    Success::ok(SubscriptionPlansResponse { plans })
}

/// Creates a new subscription checkout session for the authenticated user.
///
/// # Input
/// - `claims`: JWT claims containing user identification and Stripe customer ID
/// - `req`: JSON payload with subscription details including:
///   - `price_id`: Stripe price ID for the chosen plan
///   - `success_url`: URL to redirect after successful checkout
///   - `cancel_url`: URL to redirect if user cancels checkout
/// - `config`: Application configuration with Stripe API credentials
///
/// # Output
/// - Success: Returns a JSON object with a URL to the Stripe Checkout session
/// - Error: Returns appropriate error responses for various failure scenarios
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API
/// const response = await fetch('/api/secured/sub/subscribe', {
///   method: 'POST',
///   headers: {
///     'Content-Type': 'application/json',
///     'Authorization': `Bearer ${localStorage.getItem('authToken')}`
///   },
///   body: JSON.stringify({
///     price_id: "price_1234567890", // From the available plans endpoint
///     success_url: "https://yourapp.com/subscription/success",
///     cancel_url: "https://yourapp.com/subscription/canceled"
///   })
/// });
///
/// if (response.ok) {
///   const data = await response.json();
///   // Redirect the user to the Stripe Checkout page
///   window.location.href = data.url;
/// }
/// ```
#[post("/subscribe")]
pub async fn post_subscribe(
    claims: web::ReqData<Claims>,
    req: web::Json<SubscriptionCreateRequest>,
    config: web::Data<Arc<Config>>,
) -> impl Responder {
    let client = stripe::create_client(&config.stripe_secret_key);
    let customer = services::pay::get_customer(&client, &claims.stripe_customer_id).await?;

    let stripe_req = SubscriptionRequest {
        price_id: req.price_id.clone(),
        success_url: req.success_url.clone(),
        cancel_url: req.cancel_url.clone(),
    };

    let session =
        services::pay::create_subscription_session(&client, &customer, stripe_req).await?;

    Success::created(SubscriptionResponse {
        url: session.url.unwrap_or_else(|| "".to_string()),
    })
}

/// Creates a new enterprise subscription with custom pricing.
///
/// # Input
/// - `claims`: JWT claims containing user identification and Stripe customer ID
/// - `req`: JSON payload with enterprise subscription details:
///   - `name`: Name for the custom enterprise plan
///   - `amount`: Price in cents
///   - `interval`: Billing interval (month, year)
///   - `success_url`: URL to redirect after successful checkout
///   - `cancel_url`: URL to redirect if user cancels checkout
/// - `config`: Application configuration with Stripe API credentials
///
/// # Output
/// - Success: Returns a JSON object with a URL to the Stripe Checkout session
/// - Error: Returns appropriate error responses for various failure scenarios
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API
/// const response = await fetch('/api/secured/sub/enterprise', {
///   method: 'POST',
///   headers: {
///     'Content-Type': 'application/json',
///     'Authorization': `Bearer ${localStorage.getItem('authToken')}`
///   },
///   body: JSON.stringify({
///     name: "Enterprise Plan - Custom",
///     amount: 99900, // $999.00 in cents
///     interval: "month",
///     success_url: "https://yourapp.com/subscription/success",
///     cancel_url: "https://yourapp.com/subscription/canceled"
///   })
/// });
///
/// if (response.ok) {
///   const data = await response.json();
///   // Redirect the user to the Stripe Checkout page
///   window.location.href = data.url;
/// }
/// ```
#[post("/enterprise")]
pub async fn post_enterprise(
    claims: web::ReqData<Claims>,
    req: web::Json<EnterpriseSubscriptionRequest>,
    config: web::Data<Arc<Config>>,
) -> impl Responder {
    let client = stripe::create_client(&config.stripe_secret_key);
    let customer = services::pay::get_customer(&client, &claims.stripe_customer_id).await?;

    let session =
        services::sub::create_enterprise_subscription(&client, &customer, req.into_inner()).await?;

    Success::created(SubscriptionResponse {
        url: session.url.unwrap_or_else(|| "".to_string()),
    })
}

/// Retrieves the authenticated user's current subscription information from Stripe.
///
/// # Input
/// - `claims`: JWT claims containing user identification and Stripe customer ID
/// - `config`: Application configuration with Stripe API credentials
///
/// # Output
/// - Success: Returns a JSON object with the user's subscription details
/// - Error: Returns 404 Not Found if no subscription exists
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API with authorization
/// const response = await fetch('/api/secured/sub/current', {
///   headers: {
///     'Authorization': `Bearer ${localStorage.getItem('authToken')}`
///   }
/// });
///
/// if (response.ok) {
///   const data = await response.json();
///   console.log('Current subscription:', data.subscription);
///   // Example response:
///   // {
///   //   subscription: {
///   //     id: "sub_123abc",
///   //     customer_id: "cus_456def",
///   //     price_id: "price_789ghi",
///   //     status: "active",
///   //     current_period_end: 1672531200, // Unix timestamp
///   //     cancel_at_period_end: false
///   //   }
///   // }
/// } else if (response.status === 404) {
///   console.log('No active subscription found');
///   // Show subscription options to the user
/// }
/// ```
#[get("/current")]
pub async fn get_current(
    claims: web::ReqData<Claims>,
    config: web::Data<Arc<Config>>,
) -> impl Responder {
    let client = stripe::create_client(&config.stripe_secret_key);
    let subscription = services::sub::get_user_subscription(&client, &claims.stripe_customer_id)
        .await?
        .ok_or_else(|| AppError::NotFound("No active subscription found".to_string()))?;

    Success::ok(UserSubscriptionResponse { subscription })
}

/// Updates the auto-renewal setting for the user's current subscription.
///
/// # Input
/// - `claims`: JWT claims containing user identification and Stripe customer ID
/// - `req`: JSON payload with auto-renewal setting:
///   - `auto_renew`: Boolean indicating whether the subscription should auto-renew
/// - `config`: Application configuration with Stripe API credentials
///
/// # Output
/// - Success: Returns a JSON object with the updated subscription details
/// - Error: Returns 404 Not Found if no subscription exists
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API with authorization
/// const response = await fetch('/api/secured/sub/auto-renew', {
///   method: 'POST',
///   headers: {
///     'Content-Type': 'application/json',
///     'Authorization': `Bearer ${localStorage.getItem('authToken')}`
///   },
///   body: JSON.stringify({
///     auto_renew: true // Set to false to disable auto-renewal
///   })
/// });
///
/// if (response.ok) {
///   const data = await response.json();
///   console.log('Updated subscription:', data.subscription);
/// }
/// ```
#[post("/auto-renew")]
pub async fn post_auto_renew(
    claims: web::ReqData<Claims>,
    req: web::Json<UpdateAutoRenewRequest>,
    config: web::Data<Arc<Config>>,
) -> impl Responder {
    let client = stripe::create_client(&config.stripe_secret_key);

    // verify the subscription belongs to this user
    let subscription = services::sub::get_user_subscription(&client, &claims.stripe_customer_id)
        .await?
        .ok_or_else(|| AppError::NotFound("No active subscription found".to_string()))?;

    // update the subscription
    let updated_subscription =
        services::sub::update_subscription_auto_renew(&client, &subscription.id, req.auto_renew)
            .await?;

    Success::ok(UserSubscriptionResponse {
        subscription: updated_subscription,
    })
}
