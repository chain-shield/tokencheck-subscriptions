# Logger Module

This module provides logging functionalities for the Actix Web application, including request logging, response logging, and persistent logging to a file and database.

## Module Structure

```
logger/
├── middleware.rs  # Actix Web middleware for request/response logging
└── mod.rs         # Logger setup and initialization
```

## Usage

1.  **Logger Setup:**
    * Call the `logger::setup()` function in `main.rs` to initialize logging.
    * Example: `logger::setup().expect("Failed to set up logger");`

2.  **Logger Middleware:**
    * Wrap the Actix Web application with `logger::middleware(console_logging_enabled)` to enable request and response logging.
    * Example: `.wrap(logger::middleware(logger_enabled))`

3.  **Database Logging:**
    * Ensure the database connection pool is available in the request extensions.
    * Log entries are automatically inserted into the database by the middleware using `db::log::insert_log`.
    * The log entries include request method, path, status code, user ID (if authenticated), request/response bodies, and client information.

4.  **Console Logging:**
    * Configure the `console_logging_enabled` flag when creating the middleware.
    * When enabled, requests and responses are logged to the console with color-coded output.

5.  **File Logging:**
    * Log entries are automatically written to the `snipper.log` file.
    * The log file includes timestamps, log levels, and module information.