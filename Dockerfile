FROM rust:slim-bookworm AS builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y musl-tools ca-certificates
RUN rustup target add x86_64-unknown-linux-musl

COPY . .

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.21

RUN apk add --no-cache ca-certificates

COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/vpm-oblivius /usr/local/bin/vpm-oblivius

WORKDIR /app

CMD ["vpm-oblivius"]
