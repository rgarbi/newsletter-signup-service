-- Add migration script here
ALTER TABLE subscriptions
    ADD COLUMN stripe_subscription_id TEXT NOT NULL UNIQUE DEFAULT '_blank';
CREATE UNIQUE INDEX stripe_subscription_id_idx ON subscriptions (stripe_subscription_id);