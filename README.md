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

# End-To-End tests

In order to run the end-to-end tests, you need to generate certificates.

1. Go into `scripts`
2. Generate certificates by running the following

```zsh
./generate_cert.sh ../e2e/cert
```

# Changing the database queries for SqliteStore.

If you need to edit the database queries for SqliteStore, you must first instal sqlx-cli:

```
cargo install sqlx-cli
```

Then, create a .env file pointing to a Sqlite database file:

```
~/path/to/project/client/database/dev.db
```

Then, to create the file if it does not exist yet, type the following:

```
sqlx db create
```

and then:

```
sqlx migrate run
```

Once this is done, you can edit the queries. When you are done, remember to run:

```
cargo sqlx prepare --workspace
```

from the project root.
