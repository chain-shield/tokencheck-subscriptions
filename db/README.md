# Database Module

This module provides database interaction functionalities using `sqlx` for PostgreSQL. It includes setup, migration, and data access operations for user and log data.

### Database Setup (`setup`)

* Initializes a PostgreSQL database connection pool using `sqlx::PgPool`.
* Creates the database if it doesn't exist.
* Runs database migrations using `sqlx::migrate!`.
* Requires `DATABASE_URL` as an environment variable.
* Requires a boolean `require_ssl` to enable or disable SSL.
* Returns an `Arc<PgPool>` for use throughout the application.

## Models and DTOs

* The `models` module defines the database entities as Rust structs.
* The `dtos` module defines data transfer objects for API interactions, separating the database schema from the API contract.

## Migrations

* The `migrations` directory contains SQL files for database schema migrations.
* Migrations are automatically applied during database setup using `sqlx::migrate!`.