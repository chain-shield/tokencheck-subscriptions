-- First, drop the primary key on api_usage_daily and recreate it without plan_id
ALTER TABLE api_usage_daily DROP CONSTRAINT api_usage_daily_pkey;
ALTER TABLE api_usage_daily ADD PRIMARY KEY (user_id, date);

-- Remove plan_id columns from API tables
ALTER TABLE api_usage DROP COLUMN plan_id;
ALTER TABLE api_usage_daily DROP COLUMN plan_id;

-- Now drop the subscription tables
DROP TABLE user_subscriptions;
DROP TABLE subscription_plans;