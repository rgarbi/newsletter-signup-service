-- Add migration script here
ALTER TABLE subscriptions
    ADD COLUMN subscription_cancelled_on_date timestamptz;

ALTER TABLE subscriptions
    ADD COLUMN subscription_anniversary_day int NOT NULL;

ALTER TABLE subscriptions
    ADD COLUMN subscription_anniversary_month int NOT NULL;
