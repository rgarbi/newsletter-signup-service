-- Add migration script here
CREATE TABLE subscribers(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  email_address TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL
);
