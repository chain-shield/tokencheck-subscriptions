# Extractor Module

This module provides middleware for extracting authentication information from incoming requests. It supports both JWT tokens and API keys.

## Middleware

### 1. `ExtractionMiddleware`

*   **Purpose:** Extracts and validates authentication information from request headers.
*   **Functionality:**
    *   Retrieves JWT token from the `Authorization` header.
    *   Retrieves API key from the `X-API-KEY` header.
    *   Validates the JWT token and API key.
    *   Inserts the extracted claims into the request extensions for future use.
*   **Usage:** Applied to Actix Web routes using `app.wrap(middleware())`.
*   **Algorithm:**
    1.  **Retrieve Headers:**
        *   Retrieves the `Authorization` header for JWT token.
        *   Retrieves the `X-API-KEY` header for API key.
    2.  **Validate JWT Token:**
        *   If a JWT token is present, it is validated using the configured JWT secret.
        *   The extracted claims are inserted into the request extensions.
    3.  **Validate API Key:**
        *   If an API key is present, it is parsed and validated.
        *   The extracted claims are inserted into the request extensions.
    4.  **Forward Request:**
        *   The request is forwarded to the next service in the chain.
