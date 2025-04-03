-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(100) NOT NULL UNIQUE,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    company_name VARCHAR(100),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    verification_origin VARCHAR(100) NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT FALSE 
);

-- Table for credentials-based authentication
CREATE TABLE auth_credentials (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL
);

-- Table for external providers (Google, GitHub, etc.)
CREATE TABLE auth_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,          -- e.g., 'google', 'github'
    provider_user_id VARCHAR(255) NOT NULL, -- ID assigned by the provider
    UNIQUE (provider, provider_user_id)     -- Prevent duplicate entries
);