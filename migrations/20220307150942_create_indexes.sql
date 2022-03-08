-- Add migration script here
CREATE UNIQUE INDEX email_idx ON subscribers (email_address);
