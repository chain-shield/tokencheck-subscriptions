use std::str::FromStr;

use crate::{
    common::error::{AppError, Res},
    db::models::log::Log,
};
use sqlx::{types::ipnetwork::Ipv4Network, PgPool};
use stripe::{
    CheckoutSession, Client, CreateProduct, Customer, CustomerId, Expandable, ListPrices, Price,
    Product, Subscription,
};
use uuid::Uuid;

use crate::api_subs::{
    dtos::{
        pay::{CustomSubscriptionRequest, RecurringInfo},
        sub::EnterpriseSubscriptionRequest,
    },
    models::sub::{SubscriptionPlan, UserSubscription},
};

/// Gets a list of subscription plans.
pub async fn get_subscription_plans(client: &Client) -> Res<Vec<SubscriptionPlan>> {
    let params = ListPrices {
        active: Some(true),
        limit: Some(100),
        expand: &["data.product"],
        ..Default::default()
    };

    let prices = Price::list(client, &params).await.map_err(AppError::from)?;

    let plans = prices
        .data
        .into_iter()
        .filter_map(|price| {
            // Only include subscription prices
            if price.type_ != Some(stripe::PriceType::Recurring) {
                return None;
            }

            let product = match price.product {
                // if it's been fully expanded *and* `active == true`, keep it
                Some(Expandable::Object(prod)) if prod.active.unwrap_or(false) => prod,
                // everything else (not expanded, archived, or missing) gets filtered out
                _ => return None,
            };

            let recurring = price.recurring?;

            let features: Option<Vec<String>> = product
                .metadata
                .clone()
                .unwrap_or_default()
                .get("features")
                .and_then(|s| serde_json::from_str(s).ok());

            Some(SubscriptionPlan {
                id: price.id.to_string(),
                name: product.name.clone().unwrap_or_default(),
                description: product.description.clone().unwrap_or_default(),
                price: price.unit_amount.unwrap_or(0),
                currency: price.currency.unwrap_or_default().to_string(),
                interval: recurring.interval.to_string(),
                active: true,
                features,
                metadata: serde_json::to_value(&product.metadata).ok(),
            })
        })
        .collect();

    Ok(plans)
}

pub async fn cancel_user_account(
    client: &Client,
    pool: &PgPool,
    user_id: &Uuid,
    customer_id_str: &str,
) -> Res<()> {
    // 0) Store request metadata in app state before user deletion
    // This step depends on how your app is structured
    // If you have access to a request ID or can set attributes on the request, do it here

    // 1) Parse the Stripe customer ID
    let customer_id: CustomerId = customer_id_str.parse().map_err(|e| {
        AppError::Internal(format!("Invalid customer id: {}: {}", customer_id_str, e))
    })?;

    // 2) List all subscriptions for that customer
    let subs = Subscription::list(
        client,
        &stripe::ListSubscriptions {
            customer: Some(customer_id.clone()),
            status: None, // cancel even inactive if you like, or `Some(All)`
            limit: None,
            ..Default::default()
        },
    )
    .await
    .map_err(AppError::from)?;

    // 3) Immediately cancel each one
    for s in subs.data {
        Subscription::cancel(client, &s.id, stripe::CancelSubscription::new())
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to cancel subscription {}: {}", s.id, e))
            })?;
    }

    // 4) Delete the Stripe customer entirely
    let _ = Customer::delete(client, &customer_id).await.map_err(|e| {
        AppError::Internal(format!("Failed to delete customer {}: {}", customer_id, e))
    })?;

    // 5) Important: First log the account deletion as a special event
    // This creates a log entry BEFORE the user is deleted
    let log_entry = Log {
        id: Uuid::nil(), // auto-generated
        timestamp: chrono::Utc::now().naive_utc(),
        method: "DELETE".to_string(),
        path: "/api/secured/sub/cancel".to_string(), // Adjust path if needed
        status_code: 200,
        user_id: Some(*user_id), // Still valid as user exists
        params: Some(serde_json::json!({})),
        request_body: Some(serde_json::json!({
            "action": "account_deletion",
            "customer_id": customer_id_str
        })),
        response_body: Some(serde_json::json!({"success": true})),
        ip_address: sqlx::types::ipnetwork::IpNetwork::from_str(&"0.0.0.0".to_string())
            .expect("invalid ip"),
        user_agent: "API internal call".to_string(),
    };

    crate::db::log::insert_log(pool, log_entry)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to log account deletion: {}", e)))?;

    // Begin transaction for database operations
    let mut tx = pool.begin().await.map_err(AppError::from)?;

    // 6) Delete any logs for this user - NO LONGER NEEDED since we're modifying the logger
    // We want to keep logs but need to handle post-deletion logging

    // 7) Delete any other tables with foreign key relationships to users
    // Example (adapt as needed for your schema):
    // sqlx::query!(r#"DELETE FROM user_sessions WHERE user_id = $1"#, user_id)
    //    .execute(&mut *tx)
    //    .await
    //    .map_err(AppError::from)?;

    // 8) Delete the user
    sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, user_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;

    // Commit the transaction
    tx.commit().await.map_err(AppError::from)?;

    // 9) Set a request-specific flag to prevent logging with user_id
    // There are several ways to implement this:
    // a) If you're using request extensions, set a flag
    // b) Return a custom response struct with metadata
    // c) Use a thread-local or context variable

    // For example, if using actix-web extensions:
    // req.extensions_mut().insert(SkipUserIdLogging(true));

    // For now, we'll return normally and fix the logger middleware
    Ok(())
}

/// Gets customer's subscription.
/// Returns None if customer is not subscribed to any plan.
pub async fn get_user_subscription(
    client: &Client,
    customer_id: &str,
) -> Res<Option<UserSubscription>> {
    let customer_id = customer_id
        .parse::<CustomerId>()
        .map_err(|e| AppError::Internal(format!("Invalid customer ID: {}", e)))?;

    let subscriptions = Subscription::list(
        client,
        &stripe::ListSubscriptions {
            customer: Some(customer_id.clone()),
            status: Some(stripe::SubscriptionStatusFilter::Active),
            limit: Some(1),
            ..Default::default()
        },
    )
    .await
    .map_err(AppError::from)?;

    if let Some(sub) = subscriptions.data.first() {
        let user_sub = UserSubscription {
            id: sub.id.to_string(),
            customer_id: customer_id.to_string(),
            price_id: sub
                .items
                .data
                .first()
                .map(|item| item.price.clone().unwrap().id.to_string())
                .unwrap_or_default(),
            status: sub.status.to_string(),
            current_period_end: sub.current_period_end,
            cancel_at_period_end: sub.cancel_at_period_end,
        };
        Ok(Some(user_sub))
    } else {
        Ok(None)
    }
}

/// Creates Enterprise subscription.
pub async fn create_enterprise_subscription(
    client: &Client,
    customer: &Customer,
    req: EnterpriseSubscriptionRequest,
) -> Res<CheckoutSession> {
    // create a custom product for this enterprise plan
    let product_name = format!("Enterprise Plan: {}", req.name);
    let create_product_params = CreateProduct::new(&product_name);
    let product = Product::create(client, create_product_params)
        .await
        .map_err(AppError::from)?;

    // create the subscription session
    let custom_req = CustomSubscriptionRequest {
        product_id: product.id.to_string(),
        amount: req.amount,
        recurring_info: Some(RecurringInfo {
            interval: req.interval,
            interval_count: 1,
        }),
        success_url: req.success_url,
        cancel_url: req.cancel_url,
    };

    super::pay::create_custom_subscription_session(client, customer, custom_req).await
}

/// Update if the given subscription should be renewed
pub async fn update_subscription_auto_renew(
    client: &Client,
    subscription_id: &str,
    auto_renew: bool,
) -> Res<UserSubscription> {
    // parse the subscription ID
    let sub_id = subscription_id
        .parse::<stripe::SubscriptionId>()
        .map_err(|e| AppError::BadRequest(format!("Invalid subscription ID: {}", e)))?;

    // set cancel_at_period_end to the opposite of auto_renew (Stripe terminology)
    let cancel_at_period_end = !auto_renew;

    // call Stripe API to update the subscription
    let subscription = stripe::Subscription::update(
        client,
        &sub_id,
        stripe::UpdateSubscription {
            cancel_at_period_end: Some(cancel_at_period_end),
            ..Default::default()
        },
    )
    .await
    .map_err(AppError::from)?;

    // convert to our model
    let user_sub = UserSubscription {
        id: subscription.id.to_string(),
        customer_id: match &subscription.customer {
            stripe::Expandable::Id(id) => id.to_string(),
            stripe::Expandable::Object(customer) => customer.id.to_string(),
        },
        price_id: subscription
            .items
            .data
            .first()
            .map(|item| item.price.clone().unwrap().id.to_string())
            .unwrap_or_default(),
        status: subscription.status.to_string(),
        current_period_end: subscription.current_period_end,
        cancel_at_period_end: subscription.cancel_at_period_end,
    };

    Ok(user_sub)
}
