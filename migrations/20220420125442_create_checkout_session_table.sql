-- Add migration script here
CREATE TABLE checkout_session
(
    id                uuid PRIMARY KEY,
    user_id           TEXT        NOT NULL,
    session_state     TEXT        NOT NULL,
    created_at        timestamptz NOT NULL,
    price_lookup_key  TEXT        NOT NULL,
    stripe_session_id TEXT        NOT NULL,
    subscription      jsonb       NOT NULL
);

CREATE INDEX user_id_idx ON checkout_session (user_id);
CREATE INDEX stripe_session_id_idx ON checkout_session (stripe_session_id);
