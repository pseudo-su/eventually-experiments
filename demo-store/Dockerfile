# https://shaneutt.com/blog/rust-fast-small-docker-image-builds/

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:latest as cargo-build

RUN apt-get update

RUN apt-get install musl-tools -y

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/demo-store

COPY Cargo.toml Cargo.toml

RUN mkdir src

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/lib.rs

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/demo-store*

COPY . .

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

RUN addgroup -g 1000 appuser

RUN adduser -D -s /bin/sh -u 1000 -G appuser appuser

WORKDIR /home/appuser/bin/

COPY --from=cargo-build /usr/src/demo-store/target/x86_64-unknown-linux-musl/release/demo-store .

RUN chown appuser:appuser demo-store

USER appuser

CMD ["./demo-store"]
