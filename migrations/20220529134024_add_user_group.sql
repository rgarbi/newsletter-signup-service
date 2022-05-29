-- Add migration script here
ALTER TABLE users
    ADD COLUMN user_group TEXT NOT NULL DEFAULT 'USER';