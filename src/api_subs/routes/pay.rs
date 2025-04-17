use std::sync::Arc;

use actix_web::{Responder, get, post, web};
use crate::common::{env_config::Config, error::{AppError, Res}, http::Success, jwt::Claims};
use crate::common::stripe;
use crate::api_subs::{dtos::pay::{PaymentIntentsRequest, PaymentIntentsResponse, RefundRequest, RefundResponse}, services};

/// Handles Stripe webhook events for payment processing.
///
/// # Input
/// - `payload`: Raw string containing the webhook event data
/// - `req`: HTTP request containing Stripe signature in headers
/// - `config`: Application configuration with webhook secret
///
/// # Output
/// - Success: Returns 200 OK when webhook is processed successfully
/// - Error: Returns 400 Bad Request for invalid signature or 500 for processing errors
///
/// # Note
/// This endpoint is not called directly from your frontend application.
/// It's called by Stripe's servers when events occur (like successful payments).
/// You'll need to configure this URL in your Stripe Dashboard under Webhooks.
///
/// # Stripe Configuration Example
/// 1. Go to Stripe Dashboard → Developers → Webhooks
/// 2. Add Endpoint: https://yourapp.com/api/pay/webhook
/// 3. Select events to listen for (payment_intent.succeeded, etc.)
/// 4. Get the webhook signing secret and set it in your environment as STRIPE_WEBHOOK_SECRET
///
/// # Example Event Types Handled
/// - payment_intent.succeeded: Processed when a payment is successful
/// - checkout_session.completed: Processed when a checkout session is completed
/// - customer.subscription.created: Processed when a new subscription is created
/// - customer.subscription.updated: Processed when a subscription is updated
/// - customer.subscription.deleted: Processed when a subscription is canceled
#[post("/webhook")]
async fn post_webhook(
    payload: String,
    req: actix_web::HttpRequest,
    config: web::Data<Arc<Config>>,
) -> Res<impl Responder> {
    let signature = match req.headers().get("stripe-signature") {
        Some(signature) => signature.to_str().unwrap_or(""),
        None => return Err(AppError::BadRequest("Stripe signature missing".to_string())),
    };

    let event = services::pay::construct_event(&payload, signature, &config.stripe_webhook_secret)?;
    services::pay::process_webhook_event(event)?;

    Success::ok("Webhook processed successfully")
}

/// Processes a refund for a payment.
///
/// # Input
/// - `claims`: JWT claims containing user authentication information
/// - `req`: JSON payload containing refund details:
///   - `payment_intent_id`: The Stripe payment intent ID to refund
///   - `amount`: (Optional) Amount to refund in cents, refunds entire payment if omitted
///   - `reason`: (Optional) Reason for refund ("duplicate", "fraudulent", "requested_by_customer")
/// - `config`: Application configuration with Stripe API credentials
///
/// # Output
/// - Success: Returns a JSON object with refund details
/// - Error: Returns 400 Bad Request for invalid data or 500 for processing errors
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API
/// const response = await fetch('/api/secured/pay/refund', {
///   method: 'POST',
///   headers: {
///     'Content-Type': 'application/json',
///     'Authorization': `Bearer ${localStorage.getItem('authToken')}`
///   },
///   body: JSON.stringify({
///     payment_intent_id: "pi_1234567890",
///     amount: 1000, // Optional: refund $10.00
///     reason: "requested_by_customer" // Optional
///   })
/// });
///
/// if (response.ok) {
///   const refundData = await response.json();
///   console.log('Refund processed:', refundData);
///   // Example response:
///   // {
///   //   id: "re_123456",
///   //   amount: 1000,
///   //   status: "succeeded",
///   //   payment_intent_id: "pi_1234567890"
///   // }
/// }
/// ```
#[post("/refund")]
async fn post_refund(
    _claims: web::ReqData<Claims>,
    req: web::Json<RefundRequest>,
    config: web::Data<Arc<Config>>,
) -> Res<impl Responder> {
    let client = stripe::create_client(&config.stripe_secret_key);

    let refund = services::pay::process_refund(&client, &req).await?;

    let response = RefundResponse {
        id: refund.id.to_string(),
        amount: refund.amount,
        status: refund.status.unwrap_or_default().to_string(),
        payment_intent_id: match &refund.payment_intent {
            Some(payment_intent) => payment_intent.id().to_string(),
            None => String::new(),
        },
    };

    Success::ok(response)
}

/// Retrieves payment information for a subscription, including the payment intent ID
/// needed for refund operations.
///
/// # Input
/// - `claims`: JWT claims containing user authentication information
/// - `subscription_id`: Path parameter with the subscription ID to lookup
/// - `config`: Application configuration with Stripe API credentials
///
/// # Output
/// - Success: Returns payment details including payment_intent_id
/// - Error: Returns 404 if no payment is found or 403 if user isn't authorized
#[get("/subscription-payment/{subscription_id}")]
async fn get_subscription_payment(
    claims: web::ReqData<Claims>,
    path: web::Path<String>,
    config: web::Data<Arc<Config>>,
) -> Res<impl Responder> {
    let subscription_id = path.into_inner();
    let client = stripe::create_client(&config.stripe_secret_key);

    // Get payment information for the subscription
    let payment_info = services::pay::get_subscription_payment(
        &client,
        &subscription_id,
        &claims.stripe_customer_id,
    )
    .await?;

    Success::ok(payment_info)
}

/// Retrieves payment intents for the authenticated user with optional pagination.
///
/// # Input
/// - `claims`: JWT claims containing user authentication information and Stripe customer ID
/// - `req`: Optional JSON payload with query parameters:
///   - `user_id`: Optional customer ID
///   - `limit`: Optional limit on number of results (default: 25, max: 100)
///   - `ending_before`: Optional cursor for pagination (exclusive)
///   - `starting_after`: Optional cursor for pagination (exclusive)
/// - `config`: Application configuration with Stripe API credentials
///
/// # Output
/// - Success: Returns a JSON object with:
///   - `payment_intents`: Array of payment intents matching the query parameters
///   - `has_more`: Boolean indicating if more results are available
/// - Error: Returns appropriate error responses for various failure scenarios
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API with pagination
/// const response = await fetch('/api/secured/pay/payment-intents', {
///   method: 'POST',
///   headers: {
///     'Content-Type': 'application/json',
///     'Authorization': `Bearer ${localStorage.getItem('authToken')}`
///   },
///   body: JSON.stringify({
///     limit: 10,
///     starting_after: "pi_lastSeenId" // for pagination
///   })
/// });
///
/// if (response.ok) {
///   const data = await response.json();
///   console.log('Payment intents:', data.payment_intents);
///   console.log('Has more results:', data.has_more);
/// }
/// ```
#[post("/payment-intents")]
async fn post_payment_intents(
    claims: web::ReqData<Claims>,
    req: web::Json<PaymentIntentsRequest>,
    config: web::Data<Arc<Config>>,
) -> Res<impl Responder> {
    let client = stripe::create_client(&config.stripe_secret_key);

    // determine which customer ID to use
    let customer_id = match &req.user_id {
        Some(user_id) => user_id.clone(),
        _ => claims.stripe_customer_id.clone(),
    };

    let payment_intents =
        services::pay::get_customer_payment_intents(&client, &customer_id, &req).await?;

    Success::ok(PaymentIntentsResponse {
        intents: payment_intents,
    })
}
