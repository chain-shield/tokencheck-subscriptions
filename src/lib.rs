// Main library file for the refactored application
// This replaces the previous workspace structure

// API Subscriptions module
pub mod api_subs {
    pub mod routes {
        pub mod pay;
        pub mod server_calls;
        pub mod sub;
    }

    pub mod services {
        pub mod pay;
        pub mod sub;
    }

    pub mod dtos {
        pub mod pay;
        pub mod sub;
    }

    pub mod models {
        pub mod sub;
    }

    pub mod misc {
        pub mod pay;
    }

    // Re-export mount functions
    pub use crate::api_subs::routes::*;
    pub mod mount;
}

// Auth module
pub mod auth {
    pub mod middleware {
        pub mod auth;
        pub mod validate_api_key;
    }

    pub mod services {
        pub mod auth_client;
    }

    // Re-export auth middleware
    pub use middleware::auth::AuthMiddleware;
}

// Common utilities module
pub mod common {
    pub mod env_config;
    pub mod error;
    pub mod http;
    pub mod jwt;
    pub mod misc;
    pub mod stripe;
}

// Database module
pub mod db {
    pub mod log;

    pub mod models {
        pub mod log;
    }
}

// Logger module
pub mod logger;

// Re-export commonly used items for convenience
pub use common::error::AppError;
pub use common::http::Success;
pub use common::jwt::Claims;
