-- Add migration script here
ALTER TABLE subscriptions ALTER COLUMN subscription_mailing_address_line_1 SET NOT NULL;
ALTER TABLE subscriptions ALTER COLUMN subscription_mailing_address_line_2 SET NOT NULL;
ALTER TABLE subscriptions ALTER COLUMN subscription_city SET NOT NULL;
ALTER TABLE subscriptions ALTER COLUMN subscription_state SET NOT NULL;
ALTER TABLE subscriptions ALTER COLUMN subscription_postal_code SET NOT NULL;
ALTER TABLE subscriptions ALTER COLUMN subscription_email_address SET NOT NULL;



