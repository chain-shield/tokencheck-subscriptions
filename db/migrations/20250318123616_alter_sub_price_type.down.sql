-- Change price column from REAL back to DECIMAL(10,2)
ALTER TABLE subscription_plans ALTER COLUMN price TYPE DECIMAL(10,2);