CREATE TABLE accounts (
    id                INTEGER PRIMARY KEY NOT NULL GENERATED ALWAYS AS IDENTITY,
    account_id        UUID NOT NULL UNIQUE,
    username          VARCHAR(36) NOT NULL UNIQUE,
    identity_key      BYTEA NOT NULL
);

CREATE TABLE devices (
    id              INTEGER PRIMARY KEY NOT NULL GENERATED ALWAYS AS IDENTITY,
    owner           INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id       TEXT NOT NULL,
    name            TEXT NOT NULL,
    auth_token      TEXT NOT NULL,
    salt            TEXT NOT NULL,
    registration_id TEXT NOT NULL,
    UNIQUE(owner, device_id)
);

CREATE TABLE msq_queue (
    id          INTEGER PRIMARY KEY NOT NULL GENERATED ALWAYS AS IDENTITY,
    receiver    INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    msg         BYTEA NOT NULL
);

CREATE TABLE ec_pre_key_store (
    id          INTEGER PRIMARY KEY NOT NULL GENERATED ALWAYS AS IDENTITY,
    owner       INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    key_id      TEXT  NOT NULL,
    public_key  BYTEA NOT NULL,
    UNIQUE(owner, key_id)
);

CREATE TABLE one_time_pq_pre_key_store (
    id          INTEGER PRIMARY KEY NOT NULL GENERATED ALWAYS AS IDENTITY,
    owner       INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    key_id      TEXT NOT NULL,
    public_key  bytea NOT NULL,
    signature   bytea NOT NULL,
    UNIQUE(owner, key_id)
);

CREATE TABLE signed_pre_key_store (
    id          INTEGER PRIMARY KEY NOT NULL GENERATED ALWAYS AS IDENTITY,
    owner       INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    key_id      TEXT  NOT NULL,
    public_key  BYTEA NOT NULL,
    signature   BYTEA NOT NULL,
    UNIQUE(owner, key_id)
);

CREATE TABLE pq_last_resort_pre_key_store (
    id          INTEGER PRIMARY KEY NOT NULL GENERATED ALWAYS AS IDENTITY,
    owner       INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    key_id      TEXT  NOT NULL,
    public_key  BYTEA NOT NULL,
    signature   BYTEA NOT NULL,
    UNIQUE(owner, key_id)
);
