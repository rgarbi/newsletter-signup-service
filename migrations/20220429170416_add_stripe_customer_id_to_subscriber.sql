-- Add migration script here
ALTER TABLE subscribers
    ADD COLUMN stripe_customer_id TEXT;
