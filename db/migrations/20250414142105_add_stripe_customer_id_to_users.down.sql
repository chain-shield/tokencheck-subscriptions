-- Remove stripe_customer_id column from users table
ALTER TABLE users DROP COLUMN stripe_customer_id;