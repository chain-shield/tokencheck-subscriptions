# Limiter Module

This module provides functionalities for rate limiting and quota limiting incoming requests. It includes two types of middleware: global rate limiting and quota-based rate limiting.

## Middleware

### 1. `GlobalLimiter`

*   **Purpose:** Implements global rate limiting to protect against abuse and ensure fair usage of the API.
*   **Functionality:**
    *   Limits the number of requests per second for all users.
    *   Uses the `governor` crate for rate limiting.
    *   Returns a `429 Too Many Requests` error if the limit is reached.
*   **Usage:** Applied to the Actix Web app using `app.wrap(GlobalLimiter::new(permits_per_second))`.
*   **Algorithm:**
    *   Uses a direct rate limiter with an in-memory state and a quanta clock.
    *   The `check()` method is called for each request to determine if it should be allowed.
    *   If the request is allowed, it is passed to the next service.
    *   If the request is not allowed, a `429 Too Many Requests` error is returned.

### 2. `QuotaRateLimiter`

*   **Purpose:** Implements quota-based rate limiting based on subscription plans.
*   **Functionality:**
    *   Limits the number of requests per day and month based on the user's subscription plan.
    *   Uses Redis to store and track request counts.
    *   Returns a `429 Too Many Requests` error if the limit is reached.
*   **Usage:** Applied to the Actix Web app using `app.wrap(QuotaRateLimiter::new(plans, redis_client))`.
*   **Algorithm:**
    1.  **Find Subscription Plan:**
        *   Retrieves the subscription plan ID from the key claims.
        *   Looks up the subscription plan in the configured plans map.
    2.  **Parse Limits:**
        *   Parses the daily and monthly API limits from the subscription plan metadata.
    3.  **Get Redis Connection:**
        *   Retrieves a connection from the Redis connection pool.
    4.  **Prepare Redis Keys and TTLs:**
        *   Creates Redis keys for daily and monthly quotas based on the user ID and current date/month.
        *   Calculates the time until midnight and the end of the month to set TTLs for the Redis keys.
    5.  **Check and Increment Limits:**
        *   Increments the daily and monthly request counts in Redis.
        *   If the count exceeds the limit, decrements the count and returns a `429 Too Many Requests` error.
    6.  **Forward Request:**
        *   If the request is within the limits, it is passed to the next service.

## Helper Functions

*   `calculate_seconds_until_midnight`: Calculates the number of seconds until midnight.
*   `calculate_seconds_until_end_of_month`: Calculates the number of seconds until the end of the month.

