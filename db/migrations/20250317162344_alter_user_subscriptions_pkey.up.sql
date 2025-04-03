-- Add a new UUID column for the primary key
ALTER TABLE user_subscriptions ADD COLUMN id UUID DEFAULT gen_random_uuid();

-- Drop the old primary key
ALTER TABLE user_subscriptions DROP CONSTRAINT user_subscriptions_pkey;

-- Set the new primary key
ALTER TABLE user_subscriptions ADD PRIMARY KEY (id);

-- Ensure that a user cannot subscribe to the same plan at the exact same start date
ALTER TABLE user_subscriptions ADD CONSTRAINT unique_user_plan_start UNIQUE (user_id, plan_id, start_date);