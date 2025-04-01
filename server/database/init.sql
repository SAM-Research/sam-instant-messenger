CREATE TABLE accounts (
    id                INTEGER PRIMARY KEY NOT NULL GENERATED ALWAYS AS IDENTITY,
    account_id        UUID NOT NULL UNIQUE,
    username          VARCHAR(36) NOT NULL UNIQUE,
    identity_key      BYTEA NOT NULL
);

