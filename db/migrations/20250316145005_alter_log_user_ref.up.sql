-- Drop the foreign key constraint
ALTER TABLE logs DROP CONSTRAINT logs_user_id_fkey;

-- Add the foreign key constraint with ON DELETE CASCADE
ALTER TABLE logs ADD CONSTRAINT logs_user_id_fkey
    FOREIGN KEY (user_id)
    REFERENCES users(id)
    ON DELETE CASCADE;