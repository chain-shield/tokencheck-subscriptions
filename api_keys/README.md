# API Keys Module

This module provides API key management functionalities, including key generation, revocation, and usage tracking.

## Routes

### 1. `GET /key/keys`

*   **Purpose:** Retrieves all API keys for the authenticated user.
*   **Request Type:** `GET`
*   **Protected:** Requires a valid JWT token in the `Authorization` header.
*   **Response:**
    *   `200 OK`: Returns a JSON object containing an array of API keys.
    *   `401 Unauthorized`: If no valid token is provided.

### 2. `POST /key/generate`

*   **Purpose:** Generates a new API key for the authenticated user.
*   **Request Type:** `POST`
*   **Request Body:**

    ```json
    {
        "name": "My API Key",
        "permissions": {}
    }
    ```
*   **Protected:** Requires a valid JWT token in the `Authorization` header.
*   **Response:**
    *   `201 Created`: Returns a JSON object containing the newly generated API key.
    *   `400 Bad Request`: If the request body is invalid.
    *   `401 Unauthorized`: If no valid token is provided.

### 3. `POST /key/revoke`

*   **Purpose:** Revokes an existing API key.
*   **Request Type:** `POST`
*   **Request Body:**

    ```json
    {
        "key_id": "uuid"
    }
    ```
*   **Protected:** Requires a valid JWT token in the `Authorization` header.
*   **Response:**
    *   `200 OK`: Returns a JSON object containing the revoked API key.
    *   `400 Bad Request`: If the request body is invalid.
    *   `401 Unauthorized`: If no valid token is provided.

### 4. `GET /key/usage`

*   **Purpose:** Retrieves usage logs for a given API key.
*   **Request Type:** `GET`
*   **Query Parameters:**
    *   `user_id` (optional): The ID of the user for whom to retrieve usage logs.
    *   `key_id` (optional): The ID of the key for whom to retrieve usage logs.
    *   `limit` (required): The maximum number of logs to retrieve.
    *   `ending_before` (optional): The timestamp to end before.
    *   `starting_after` (optional): The timestamp to start after.
*   **Protected:** Requires a valid JWT token in the `Authorization` header.
*   **Response:**
    *   `200 OK`: Returns a JSON object containing an array of usage logs.
    *   `400 Bad Request`: If the request body is invalid.
    *   `401 Unauthorized`: If no valid token is provided.

## Middleware

### 1. `KeyMiddleware`

*   **Purpose:** Authenticates API requests using API keys.
*   **Functionality:**
    *   Extracts API key claims from the request.
    *   Validates the API key against the database.
    *   If the key is valid, the request is passed to the next handler.
    *   If the key is invalid, a `401 Unauthorized` error is returned.
*   **Usage:** Applied to routes that require API key authentication using `app.wrap(middleware())`.
*   **Algorithm:**
    1.  **Extract Key Claims:**
        *   Retrieves the API key from the request headers.
        *   Parses the API key into its constituent claims.
    2.  **Validate Key:**
        *   Retrieves the API key record from the database using the key ID.
        *   Verifies that the secret matches the hashed value stored in the database.
    3.  **Forward Request:**
        *   If the key is valid, the request is forwarded to the next service.
        *   If the key is invalid, an `AppError::BadRequest` is returned.
