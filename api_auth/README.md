# API Authentication Module

This module provides API authentication functionality using Actix Web and Actix Session, including user registration, login, OAuth authentication, session management, and protected routes.

## Routes

### 1. `POST /register`

* **Purpose:** Registers a new user with email and password authentication.
* **Request Type:** `POST`
* **Request Body:**
    ```json
    {
        "email": "user@example.com",
        "password": "securepassword",
        "first_name": "John",
        "last_name": "Doe",
        "company_name": "ACME Inc" // Optional
    }
    ```
* **Response:**
    * `201 Created`: User successfully registered. Returns the created user object.
    * `400 Bad Request`: If the email already exists or the request body is invalid.

### 2. `POST /login`

* **Purpose:** Authenticates a user with email and password.
* **Request Type:** `POST`
* **Request Body:**
    ```json
    {
        "email": "user@example.com",
        "password": "securepassword"
    }
    ```
* **Response:**
    * `200 OK`: User successfully logged in. Returns an auth response with JWT token and user details.
    * `401 Unauthorized`: Invalid email or password.

### 3. `GET /oauth/{provider}`

* **Purpose:** Initiates OAuth authentication flow with the specified provider (google, github, etc.).
* **Request Type:** `GET`
* **Path Parameters:**
    * `provider`: OAuth provider name.
* **Response:**
    * `302 Found`: Redirects user to the OAuth provider's authentication page.
    * `400 Bad Request`: Invalid provider name.

### 4. `GET /oauth/{provider}/callback`

* **Purpose:** Handles OAuth callback after user authenticates with the provider.
* **Request Type:** `GET`
* **Path Parameters:**
    * `provider`: OAuth provider name.
* **Query Parameters:**
    * `code`: Authorization code from the OAuth provider.
* **Response:**
    * `302 Found`: Redirects to the application callback URL with session data set (token and user).
    * Errors can occur with invalid provider, exchange code failure, or internal server errors.
    * **Note:** This endpoint is not called directly from your frontend code.

### 5. `GET /session`

* **Purpose:** Retrieves current session data for the authenticated user from session cookies.
* **Request Type:** `GET`
* **Response:**
    * `200 OK`: Returns JSON with user data and token.
    * `401 Unauthorized`: If no valid session exists.

### 6. `GET /api/secured/me`

* **Purpose:** Retrieves the current authenticated user's information.
* **Request Type:** `GET`
* **Protected:** Requires a valid JWT token in the `Authorization` header.
* **Response:**
    * `200 OK`: Returns a JSON object with the user's profile information.
    * `401 Unauthorized`: If no valid token is provided.

## Middleware

### 1. `AuthMiddleware`

* **Purpose:** Protects routes that require authentication using JWT.
* **Functionality:**
    * Checks for a valid JWT token in the `Authorization` header.
    * If a valid token exists, it extracts claims and proceeds to the route handler.
    * If no token exists or the token is invalid, it returns a `401 Unauthorized` error.
* **Usage:** Applied to routes that require authentication using `app.wrap(AuthMiddleware::new(jwt_config))`.

### 2. `SessionMiddleware`

* **Purpose:** Manages user sessions using cookies.
* **Functionality:**
    * Creates and maintains a session for each user.
    * Stores session data (token and user info) in a cookie.
    * Uses `actix-session` with `CookieSessionStore`.
    * Configuration includes setting cookie name, security, same-site policy, and domain.
* **Usage:** Applied to the Actix Web app using `app.wrap(session_middleware(cookie_secure, is_production, secret))`.