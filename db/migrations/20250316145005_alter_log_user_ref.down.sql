-- Re-add the original foreign key constraint (without ON DELETE CASCADE)
ALTER TABLE logs DROP CONSTRAINT logs_user_id_fkey;

ALTER TABLE logs ADD CONSTRAINT logs_user_id_fkey
    FOREIGN KEY (user_id)
    REFERENCES users(id);