-- Add migration script here
CREATE TABLE subscription_event_history(
    id uuid PRIMARY KEY,
    subscription_id uuid NOT NULL,
    subscription_change_event_date timestamptz NOT NULL,
    subscription_change_event_type TEXT NOT NULL,
    subscription jsonb NOT NULL
);

CREATE INDEX subscription_id_idx ON subscription_event_history (subscription_id);