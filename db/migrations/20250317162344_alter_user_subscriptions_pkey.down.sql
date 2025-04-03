-- Drop the new primary key
ALTER TABLE user_subscriptions DROP CONSTRAINT user_subscriptions_pkey;

-- Drop the unique constraint on (user_id, plan_id, start_date)
ALTER TABLE user_subscriptions DROP CONSTRAINT unique_user_plan_start;

-- Remove the newly added `id` column
ALTER TABLE user_subscriptions DROP COLUMN id;

-- Restore the original primary key on `user_id`
ALTER TABLE user_subscriptions ADD PRIMARY KEY (user_id);