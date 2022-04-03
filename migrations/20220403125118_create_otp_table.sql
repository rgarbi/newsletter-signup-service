-- Add migration script here
CREATE TABLE otp(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    user_id TEXT NOT NULL,
    one_time_passcode TEXT NOT NULL,
    issued_on timestamptz NOT NULL,
    expires_on timestamptz NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE UNIQUE INDEX one_time_passcode_idx ON otp (one_time_passcode);