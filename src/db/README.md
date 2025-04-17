# Database Module

This module provides database functionality for the application, primarily focused on logging database operations.

## Module Structure

```
db/
├── models/       # Database models
│   └── log.rs    # Log model definition
├── log.rs        # Log database operations
└── mod.rs        # Module exports and database setup
```

## Features

### Database Setup

The `db::setup` function in `mod.rs` initializes the database connection:

```rust
pub async fn setup(
    database_url: &str,
    require_ssl: bool,
) -> Result<Arc<PgPool>, Box<dyn std::error::Error>>
```

This function:
1. Parses the database URL
2. Creates a connection to the PostgreSQL server
3. Creates the database if it doesn't exist
4. Establishes a connection pool to the database
5. Returns an Arc-wrapped PgPool for thread-safe access

### Logging

The `db::log` module provides functions for logging application events to the database:

```rust
pub async fn insert_log<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    log: Log,
) -> Res<()>
```

This function inserts a log entry into the `logs` table, including:
- Timestamp
- HTTP method
- Path
- Status code
- User ID (if authenticated)
- Request/response parameters and bodies
- IP address
- User agent

### Models

The `db::models::log` module defines the `Log` struct that maps to the database schema:

```rust
pub struct Log {
    pub id: Uuid,
    pub timestamp: NaiveDateTime,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub user_id: Option<Uuid>,
    pub params: Option<JsonValue>,
    pub request_body: Option<JsonValue>,
    pub response_body: Option<JsonValue>,
    pub ip_address: IpNetwork,
    pub user_agent: String,
}
```

## Usage

The database module is primarily used by the logger middleware to store request and response information:

```rust
// In logger/middleware.rs
crate::db::log::insert_log(
    &***pool,
    Log {
        id: Uuid::nil(), // auto-generated
        timestamp: timestamp.naive_utc(),
        method,
        path,
        status_code,
        user_id,
        params: Some(params_json),
        request_body: Some(request_body),
        response_body: Some(response_body),
        ip_address,
        user_agent,
    },
).await?;
```
