-- Add migration script here
CREATE TABLE users(
    user_id uuid PRIMARY KEY,
    email_address TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);

CREATE UNIQUE INDEX email_address_idx ON users (email_address);

