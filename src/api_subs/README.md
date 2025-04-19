# API Subscriptions Module

This module provides the API endpoints and business logic for subscription and payment management using Stripe.

## Module Structure

```
api_subs/
├── dtos/         # Data transfer objects
│   ├── pay.rs    # Payment-related DTOs
│   └── sub.rs    # Subscription-related DTOs
├── misc/         # Miscellaneous utilities
│   └── pay.rs    # Payment utilities
├── models/       # Data models
│   └── sub.rs    # Subscription models
├── routes/           # API route handlers
│   ├── pay.rs        # Payment endpoints
│   ├── sub.rs        # Subscription endpoints
│   └── server_calls.rs # Server-to-server API endpoints
├── services/     # Business logic services
│   ├── pay.rs    # Payment services
│   └── sub.rs    # Subscription services
└── mount.rs      # Route mounting functions
```

## Features

### Subscription Management

The subscription management functionality includes:

1. **Listing Subscription Plans**
   - Endpoint: `GET /api/secured/sub/plans`
   - Handler: `get_plans` in `routes/sub.rs`
   - Service: `get_subscription_plans` in `services/sub.rs`

2. **Creating Subscriptions**
   - Endpoint: `POST /api/secured/sub/subscribe`
   - Handler: `post_subscribe` in `routes/sub.rs`
   - Service: `create_subscription_session` in `services/pay.rs`

3. **Enterprise Subscriptions**
   - Endpoint: `POST /api/secured/sub/enterprise`
   - Handler: `post_enterprise` in `routes/sub.rs`
   - Service: `create_enterprise_subscription` in `services/sub.rs`

4. **Getting Current Subscription**
   - Endpoint: `GET /api/secured/sub/current`
   - Handler: `get_current` in `routes/sub.rs`
   - Service: `get_user_subscription` in `services/sub.rs`

5. **Managing Auto-Renewal**
   - Endpoint: `POST /api/secured/sub/auto-renew`
   - Handler: `post_auto_renew` in `routes/sub.rs`
   - Service: `update_subscription_auto_renew` in `services/sub.rs`

### Payment Processing

The payment processing functionality includes:

1. **Processing Refunds**
   - Endpoint: `POST /api/secured/pay/refund`
   - Handler: `post_refund` in `routes/pay.rs`
   - Service: `process_refund` in `services/pay.rs`

2. **Getting Subscription Payment Details**
   - Endpoint: `GET /api/secured/pay/subscription-payment/{subscription_id}`
   - Handler: `get_subscription_payment` in `routes/pay.rs`
   - Service: `get_subscription_payment` in `services/pay.rs`

3. **Listing Payment Intents**
   - Endpoint: `POST /api/secured/pay/payment-intents`
   - Handler: `post_payment_intents` in `routes/pay.rs`
   - Service: `get_customer_payment_intents` in `services/pay.rs`

4. **Webhook Handler**
   - Endpoint: `POST /api/pay/webhook`
   - Handler: `post_webhook` in `routes/pay.rs`
   - Services: `construct_event` and `process_webhook_event` in `services/pay.rs`

### Server-to-Server API

The server-to-server API provides endpoints for other microservices to interact with the subscription system:

1. **Create Stripe Customer**
   - Endpoint: `POST /api/server/create-customer`
   - Handler: `create_customer` in `routes/server_calls.rs`
   - Service: `create_customer` in `common/stripe.rs`
   - Description: Creates a new Stripe customer with the provided first name, last name, and email
   - Authentication: None (internal API)

## Route Mounting

The `mount.rs` file provides functions to mount the API routes:

```rust
pub fn mount_secure_subs() -> actix_web::Scope
pub fn mount_subs() -> actix_web::Scope
pub fn mount_pay() -> actix_web::Scope
pub fn mount_webhook() -> actix_web::Scope
pub fn mount_server_calls() -> actix_web::Scope
```

These functions are used in `main.rs` to set up the API endpoints with the appropriate middleware and scopes.
