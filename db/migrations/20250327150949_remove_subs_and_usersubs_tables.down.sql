CREATE TABLE subscription_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    price DECIMAL(10, 2),
    daily_api_limit INT,
    monthly_api_limit INT,
    features JSONB,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_subscriptions (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES subscription_plans(id) ON DELETE CASCADE,
    start_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    end_date TIMESTAMP,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    auto_renew BOOLEAN NOT NULL DEFAULT TRUE,
    custom_api_limit INT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Now add back plan_id columns to API tables
ALTER TABLE api_usage ADD COLUMN plan_id UUID NOT NULL REFERENCES subscription_plans(id) ON DELETE CASCADE;
ALTER TABLE api_usage_daily ADD COLUMN plan_id UUID NOT NULL REFERENCES subscription_plans(id) ON DELETE CASCADE;

-- Recreate primary key on api_usage_daily to include plan_id
ALTER TABLE api_usage_daily DROP CONSTRAINT api_usage_daily_pkey;
ALTER TABLE api_usage_daily ADD PRIMARY KEY (user_id, plan_id, date);