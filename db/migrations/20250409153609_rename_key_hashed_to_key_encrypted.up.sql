-- First add the new column to avoid losing data
ALTER TABLE api_keys ADD COLUMN key_encrypted TEXT;

-- Copy data from the old column to the new one
-- The data format will be different (hashed vs encrypted),
-- so you may need to handle this conversion in your application code
-- For now, we're just preserving the structure for data migration
UPDATE api_keys SET key_encrypted = key_hashed;

-- Now make the new column NOT NULL
ALTER TABLE api_keys ALTER COLUMN key_encrypted SET NOT NULL;

-- Finally, drop the old column
ALTER TABLE api_keys DROP COLUMN key_hashed;
