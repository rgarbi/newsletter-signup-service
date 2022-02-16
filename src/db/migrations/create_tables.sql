CREATE TABLE if not exists subscribers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    email_address VARCHAR NOT NULL
);

CREATE TABLE if not exists subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subscriber_id INTEGER NOT NULL,
    subscription_first_name VARCHAR NOT NULL,
    subscription_last_name VARCHAR NOT NULL,
    subscription_mailing_address_line_1 VARCHAR NOT NULL,
    subscription_mailing_address_line_2 VARCHAR,
    subscription_city VARCHAR NOT NULL,
    subscription_state VARCHAR NOT NULL,
    subscription_postal_code VARCHAR NOT NULL,
    subscription_email_address VARCHAR NOT NULL,
    subscription_creation_date DATETIME NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 0,
    subscription_type VARCHAR NOT NULL,
    FOREIGN KEY (subscriber_id) REFERENCES subscribers
);