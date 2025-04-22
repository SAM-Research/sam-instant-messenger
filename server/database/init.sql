CREATE DOMAIN uint32 AS bigint
    CHECK (VALUE >= 0 AND VALUE <= '4294967295'::bigint);

CREATE TABLE accounts (
    id                INTEGER PRIMARY KEY NOT NULL GENERATED ALWAYS AS IDENTITY,
    account_id        UUID NOT NULL UNIQUE,
    username          TEXT NOT NULL UNIQUE,
    identity_key      BYTEA NOT NULL
);

CREATE TABLE devices (
    id              INTEGER PRIMARY KEY NOT NULL GENERATED ALWAYS AS IDENTITY,
    owner           INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id       uint32 NOT NULL,
    registration_id uint32 NOT NULL,
    name            TEXT   NOT NULL,
    hash            TEXT   NOT NULL,
    salt            TEXT   NOT NULL,
    UNIQUE(owner, device_id)
);

CREATE TABLE device_link_info (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    link_secret TEXT NOT NULL,
    provision_expire_seconds uint32 NOT NULL
);

