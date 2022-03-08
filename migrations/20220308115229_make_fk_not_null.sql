-- Add migration script here
ALTER TABLE subscriptions ALTER COLUMN subscriber_id SET NOT NULL;
