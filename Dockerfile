FROM rust:slim-bookworm AS builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y musl-tools ca-certificates
RUN rustup target add x86_64-unknown-linux-musl

# Cache dependencies: copy manifests and build a dummy crate
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && echo '' > src/lib.rs
RUN cargo build --target x86_64-unknown-linux-musl --release || true
RUN rm -rf src

# Copy real source and rebuild (only the crate itself, deps are cached)
COPY . .
RUN touch src/main.rs src/lib.rs
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.21

RUN apk add --no-cache ca-certificates

COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/vpm-oblivius /usr/local/bin/vpm-oblivius

WORKDIR /app

CMD ["vpm-oblivius"]
