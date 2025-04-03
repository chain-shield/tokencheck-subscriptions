# Logger Module

This module provides logging functionalities for the Actix Web application, including request logging, response logging, and persistent logging to a file.

## Usage

1.  **Logger Setup:**
    * Call the `logger::setup()` function in `src/lib.rs` to initialize logging.

2.  **Logger Middleware:**
    * Wrap the Actix Web application or specific routes with `logger::middleware(console_logging_enabled)` to enable request and response logging.

3.  **Database Logging:**
    * Ensure the database connection pool is available in the request extensions.
    * Log entries are automatically inserted into the database by the middleware.

4.  **Console Logging:**
    * Configure the `console_logging_enabled` flag when creating the middleware.

5.  **File Logging:**
    * Log entries are automatically written to the `snipper.log` file.