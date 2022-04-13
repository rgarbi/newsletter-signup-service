-- Add migration script here
CREATE TABLE subscriptions(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    subscriber_id uuid,
    FOREIGN KEY (subscriber_id) REFERENCES subscribers(id),
    subscription_name TEXT NOT NULL,
    subscription_mailing_address_line_1 TEXT,
    subscription_mailing_address_line_2 TEXT,
    subscription_city TEXT,
    subscription_state TEXT,
    subscription_postal_code TEXT,
    subscription_email_address TEXT,
    subscription_creation_date timestamptz NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    subscription_type TEXT NOT NULL
);
