# sam-instant-messenger

[![Rust](https://github.com/SAM-Research/sam-instant-messenger/actions/workflows/rust.yml/badge.svg)](https://github.com/SAM-Research/sam-instant-messenger/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/SAM-Research/sam-instant-messenger/graph/badge.svg?token=QYZJ65M3N1)](https://codecov.io/gh/SAM-Research/sam-instant-messenger)

# Server

Running the server can be done with:

```sh
RUST_LOG=info cargo run --bin sam-server
```

Omit the `RUST_LOG=info` if you don't want any logging

# End-To-End tests

In order to run the end-to-end tests, you need to generate certificates.

1. Go into `e2e/cert`
2. Generate certificates by running the following

```zsh
./generate_cert.sh
```