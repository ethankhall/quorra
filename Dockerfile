# syntax=docker/dockerfile:1.4
FROM rust:bullseye as chef
COPY rust-toolchain.toml rust-toolchain.toml
RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOT
#!/usr/bin/env bash
set -euxo pipefail

apt-get update
apt-get install protobuf-compiler -y
cargo install cargo-chef
EOT

WORKDIR /app

FROM chef AS planner
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target/release/deps \
    --mount=type=cache,target=/app/target/release/build \
    cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target/release/deps \
    --mount=type=cache,target=/app/target/release/build \
    cargo chef cook --release --recipe-path recipe.json
# Build application
COPY dev-null dev-null-plugin dev-null-plugin-http Cargo.toml Cargo.lock rust-toolchain.toml /app/

FROM builder as test
RUN --mount=type=cache,target=/usr/local/cargo/registry  \
    --mount=type=cache,target=/app/target/release/deps \
    --mount=type=cache,target=/app/target/release/build <<EOT
#!/usr/bin/env bash
set -euxo pipefail

cargo test
cargo fmt --check
cargo clippy
cargo +nightly udeps
EOT

FROM builder as release
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target/release/deps \
    --mount=type=cache,target=/app/target/release/build \
     cargo build --release --bin dev-null
RUN pwd
RUN ls -al target/release

FROM debian:bullseye-slim AS runtime
RUN apt-get update && apt-get install tini -y
WORKDIR /app
COPY --from=release /app/target/release/dev-null /usr/local/bin
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/dev-null"]