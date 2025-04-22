# API Subscription Module (Actix Web)

This module provides API subscription functionality using Actix Web, including handling Stripe webhooks, processing refunds, retrieving payment information, managing payment intents, and creating and updating subscriptions.

## Routes

### 1. `POST /webhook`

* **Purpose:** Handles Stripe webhook events for payment processing.
* **Request Type:** `POST`
* **Request Body:** Raw string containing the webhook event data.
* **Headers:** `stripe-signature` containing the Stripe signature.
* **Response:**
    * `200 OK`: When webhook is processed successfully.
    * `400 Bad Request`: For invalid signature.
    * `500 Internal Server Error`: For processing errors.
* **Note:** This endpoint is called by Stripe's servers, not directly from your frontend.

To forward Stripe API calls to the webhook, use this command: `stripe listen --forward-to localhost:8080/api/pay/webhook`

### 2. `POST /refund`

* **Purpose:** Processes a refund for a payment.
* **Request Type:** `POST`
* **Request Body:**
    ```json
    {
        "payment_intent_id": "pi_1234567890",
        "amount": 1000, // Optional: refund $10.00 in cents
        "reason": "requested_by_customer" // Optional
    }
    ```
* **Protected:** Requires a valid JWT token in the `Authorization` header.
* **Response:**
    * `200 OK`: Returns a JSON object with refund details.
    * `400 Bad Request`: For invalid data.
    * `500 Internal Server Error`: For processing errors.

### 3. `GET /subscription-payment/{subscription_id}`

* **Purpose:** Retrieves payment information for a subscription.
* **Request Type:** `GET`
* **Path Parameters:**
    * `subscription_id`: The subscription ID to lookup.
* **Protected:** Requires a valid JWT token in the `Authorization` header.
* **Response:**
    * `200 OK`: Returns payment details including `payment_intent_id`.
    * `404 Not Found`: If no payment is found.
    * `403 Forbidden`: If user isn't authorized.

### 4. `POST /payment-intents`

* **Purpose:** Retrieves payment intents for the authenticated user.
* **Request Type:** `POST`
* **Request Body:**
    ```json
    {
        "user_id": "cus_123456", // Optional: to retrieve payment intents for another user
        "limit": 10, // Optional: limit on number of results
        "ending_before": "pi_lastSeenId", // Optional: cursor for pagination (exclusive)
        "starting_after": "pi_lastSeenId" // Optional: cursor for pagination (exclusive)
    }
    ```
* **Protected:** Requires a valid JWT token in the `Authorization` header.
* **Response:**
    * `200 OK`: Returns a JSON object with `payment_intents` and `has_more`.
    * Error: Returns appropriate error responses for various failure scenarios.

### 5. `GET /plans`

* **Purpose:** Retrieves all available subscription plans from Stripe.
* **Request Type:** `GET`
* **Protected:** Requires a valid JWT token in the `Authorization` header.
* **Response:**
    * `200 OK`: Returns a JSON object containing an array of subscription plans.
    * `500 Internal Server Error`: If plans cannot be retrieved.

### 6. `POST /subscribe`

* **Purpose:** Creates a new subscription checkout session for the authenticated user.
* **Request Type:** `POST`
* **Request Body:**
    ```json
    {
        "price_id": "price_1234567890",
        "success_url": "[https://yourapp.com/subscription/success](https://yourapp.com/subscription/success)",
        "cancel_url": "[https://yourapp.com/subscription/canceled](https://yourapp.com/subscription/canceled)"
    }
    ```
* **Protected:** Requires a valid JWT token in the `Authorization` header.
* **Response:**
    * `201 Created`: Returns a JSON object with a URL to the Stripe Checkout session.
    * Error: Returns appropriate error responses for various failure scenarios.

### 7. `POST /enterprise`

* **Purpose:** Creates a new enterprise subscription with custom pricing.
* **Request Type:** `POST`
* **Request Body:**
    ```json
    {
        "name": "Enterprise Plan - Custom",
        "amount": 99900,
        "interval": "month",
        "success_url": "[https://yourapp.com/subscription/success](https://yourapp.com/subscription/success)",
        "cancel_url": "[https://yourapp.com/subscription/canceled](https://yourapp.com/subscription/canceled)"
    }
    ```
* **Protected:** Requires a valid JWT token in the `Authorization` header.
* **Response:**
    * `201 Created`: Returns a JSON object with a URL to the Stripe Checkout session.
    * Error: Returns appropriate error responses for various failure scenarios.

### 8. `GET /current`

* **Purpose:** Retrieves the authenticated user's current subscription information from Stripe.
* **Request Type:** `GET`
* **Protected:** Requires a valid JWT token in the `Authorization` header.
* **Response:**
    * `200 OK`: Returns a JSON object with the user's subscription details.
    * `404 Not Found`: If no subscription exists.

### 9. `POST /auto-renew`

* **Purpose:** Updates the auto-renewal setting for the user's current subscription.
* **Request Type:** `POST`
* **Request Body:**
    ```json
    {
        "auto_renew": true // Set to false to disable auto-renewal
    }
    ```
* **Protected:** Requires a valid JWT token in the `Authorization` header.
* **Response:**
    * `200 OK`: Returns a JSON object with the updated subscription details.
    * `404 Not Found`: If no subscription exists.