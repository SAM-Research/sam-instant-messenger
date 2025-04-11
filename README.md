# sam-instant-messenger

[![Rust](https://github.com/SAM-Research/sam-instant-messenger/actions/workflows/rust.yml/badge.svg)](https://github.com/SAM-Research/sam-instant-messenger/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/SAM-Research/sam-instant-messenger/graph/badge.svg?token=QYZJ65M3N1)](https://codecov.io/gh/SAM-Research/sam-instant-messenger)

# Server

Running the server can be done with:

```sh
RUST_LOG=info cargo run --bin sam-server
```

Omit the `RUST_LOG=info` if you don't want any logging

## TLS Configuration

You can configure the server with tls by providing CLI arguments with the certificates and keys or as a JSON.

Example configuration:

```jsonc
{
  "caCertPath": "rootCA.crt", // optional, enables mTLS
  "certPath": "server.crt",
  "keyPath": "server.key"
}
```

provide the config in cli arguments with:

```sh
cargo run  --bin sam-server -- --tls-config ./config.json
```

# Docker

Building the `sam-server` docker image:

```sh
docker build -t sam-server .
```

# End-To-End tests

In order to run the end-to-end tests, you need to generate certificates.

1. Go into `scripts`
2. Generate certificates by running the following

```zsh
./generate_cert.sh ../e2e/cert
```

# Database Queries

If you need to edit the database queries, you must first install sqlx-cli:

```
cargo install sqlx-cli
```

## Changing the Database Queries for SqliteStore.

Create a .env file inside the `client` directory pointing to a Sqlite database file:

```
DATABASE_URL=~/path/to/project/client/database/dev.db
```

Then, to create the file if it does not exist yet, type the following:

first:

```
cd client
```

then:

```
sqlx db create
```

and then:

```
sqlx migrate run
```

Once this is done, you can edit the queries. When you are done, remember to run:

```
cargo sqlx prepare
```

from the project root.

## Changing the Database Queries for Postgres Managers.

Create a .env file inside the `server` directory pointing to the test database:

```
DATABASE_URL=postgres://test:test@127.0.0.1:5432/sam_test_db
```

Then you need to run the test database using docker compose:

```
docker compose -f server/database/test-database.yml up
```

You can now modify the database queries. When you are done, you should run sqlx prepare so others will not need an active database connection to compile the project:

```
cd server
```

```
cargo sqlx prepare
```
