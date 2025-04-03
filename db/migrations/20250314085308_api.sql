CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hashed VARCHAR(64) NOT NULL,
    name VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used TIMESTAMP,
    permissions JSONB NOT NULL
);

CREATE TABLE api_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES subscription_plans(id) ON DELETE CASCADE,
    timestamp TIMESTAMP NOT NULL,
    endpoint TEXT NOT NULL,
    query_params JSONB,
    request_body JSONB,
    response_body JSONB,
    response_code INT NOT NULL,
    ip_address INET NOT NULL,
    user_agent VARCHAR(255) NOT NULL
);

CREATE TABLE api_usage_daily (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES subscription_plans(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    call_count INT NOT NULL DEFAULT 0,
    successful_count INT NOT NULL DEFAULT 0,
    failed_count INT NOT NULL DEFAULT 0,
    remaining_daily_count INT NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, plan_id, date)
);