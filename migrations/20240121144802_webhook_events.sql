-- Add migration script here
CREATE TABLE webhook_events(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    event_text TEXT NOT NULL,
    sent_on timestamptz NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT FALSE
);