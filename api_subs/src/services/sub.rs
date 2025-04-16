use common::error::{AppError, Res};
use stripe::{
    CheckoutSession, Client, CreateProduct, Customer, CustomerId, ListPrices, Price, Product,
    Subscription,
};

use crate::{
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

            let recurring = price.recurring?;
            let product_obj = price.product.as_ref().and_then(|p| p.as_object())?;

            let plan = SubscriptionPlan {
                id: price.id.to_string(),
                name: product_obj.name.clone().unwrap_or_default(),
                description: product_obj.description.clone().unwrap_or_default(),
                price: price.unit_amount.unwrap_or(0),
                currency: price.currency.unwrap_or_default().to_string(),
                interval: recurring.interval.to_string(),
                active: true,
                metadata: product_obj.metadata.as_ref().and_then(|map| {
                    let json_str = serde_json::to_string(map).ok()?;
                    serde_json::from_str(&json_str).ok()
                }),
            };

            Some(plan)
        })
        .collect();

    Ok(plans)
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
            sub_id: sub.id.to_string(),
            customer_id: customer_id.to_string(),
            id: sub
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
        sub_id: subscription.id.to_string(),
        customer_id: match &subscription.customer {
            stripe::Expandable::Id(id) => id.to_string(),
            stripe::Expandable::Object(customer) => customer.id.to_string(),
        },
        id: subscription
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
