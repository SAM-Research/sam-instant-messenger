FROM rust:1.86-slim AS builder
ENV SQLX_OFFLINE=true
WORKDIR /sam-instant-messenger
COPY . .

RUN apt update 
RUN apt install -y protobuf-compiler

RUN cargo build --bin sam-server --release

LABEL org.opencontainers.image.source=https://github.com/SAM-Research/sam-instant-messenger
LABEL org.opencontainers.image.description="SAM Server image"
LABEL org.opencontainers.image.licenses=MIT


FROM debian:bookworm-slim
COPY --from=builder /sam-instant-messenger/target/release/sam-server /sam-server


ENV PORT=8080

ENTRYPOINT ["/sam-server"]
EXPOSE $PORT
