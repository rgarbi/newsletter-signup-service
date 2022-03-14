-- Add migration script here
ALTER TABLE subscribers
    ADD COLUMN user_id TEXT NOT NULL UNIQUE DEFAULT '_blank';
CREATE UNIQUE INDEX user_id_idx ON subscribers (user_id);