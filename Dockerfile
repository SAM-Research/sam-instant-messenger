FROM messense/rust-musl-cross:x86_64-musl AS builder
ENV SQLX_OFFLINE=true
WORKDIR /sam-instant-messenger
COPY . .

RUN apt update 
RUN apt install -y protobuf-compiler

RUN cargo build --bin sam-server --release --target x86_64-unknown-linux-musl

LABEL org.opencontainers.image.source=https://github.com/SAM-Research/sam-instant-messenger
LABEL org.opencontainers.image.description="SAM Server image"
LABEL org.opencontainers.image.licenses=MIT


FROM scratch
COPY --from=builder /sam-instant-messenger/target/x86_64-unknown-linux-musl/release/sam-server /sam-server


ENV PORT=8080

ENTRYPOINT ["/sam-server"]
EXPOSE $PORT

