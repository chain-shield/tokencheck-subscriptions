-- Change price column from DECIMAL(10,2) to REAL (f32)
ALTER TABLE subscription_plans ALTER COLUMN price TYPE REAL;