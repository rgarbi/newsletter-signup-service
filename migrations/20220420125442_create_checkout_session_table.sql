-- Add migration script here
CREATE TABLE checkout_session
(
    id                uuid PRIMARY KEY,
    user_id           TEXT        NOT NULL,
    session_state     TEXT        NOT NULL,
    created_at        timestamptz NOT NULL,
    price_lookup_key  TEXT        NOT NULL,
    stripe_session_id TEXT UNIQUE NOT NULL,
    subscription      jsonb       NOT NULL
);

CREATE INDEX checkout_session_user_id_idx ON checkout_session (user_id);
CREATE UNIQUE INDEX stripe_session_id_idx ON checkout_session (stripe_session_id);
