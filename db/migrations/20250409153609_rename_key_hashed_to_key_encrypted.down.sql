-- First add back the original column
ALTER TABLE api_keys ADD COLUMN key_hashed VARCHAR(64);

-- Copy data from encrypted to hashed (note: this is a simplification)
-- In reality, you can't convert encrypted data back to hashed data directly
-- This is just for schema rollback purposes
UPDATE api_keys SET key_hashed = key_encrypted;

-- Make the original column NOT NULL again
ALTER TABLE api_keys ALTER COLUMN key_hashed SET NOT NULL;

-- Drop the new column
ALTER TABLE api_keys DROP COLUMN key_encrypted;
