-- Remove last_used columns since we already store that information in logs
ALTER TABLE api_keys DROP COLUMN last_used;