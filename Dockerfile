# syntax=docker/dockerfile:1.4
FROM --platform=$BUILDPLATFORM rust:bullseye as builder
ARG TARGETARCH
RUN <<EOT
#!/usr/bin/env bash
set -euxo pipefail

apt-get update
apt-get install -y protobuf-compiler g++-aarch64-linux-gnu libc6-dev-arm64-cross g++-x86-64-linux-gnu libc6-dev-amd64-cross
EOT

WORKDIR /app
COPY rust-toolchain.toml /app/rust-toolchain.toml
RUN cargo

COPY . /app/
COPY <<EOF /app/.cargo/config.toml 
[source.crates-io]
replace-with = "vendored-sources"
[source.vendored-sources]
directory = "vendor"
EOF

FROM --platform=$BUILDPLATFORM builder as release
RUN <<EOT
#!/bin/bash
set -eux
set -o pipefail
if [ "$TARGETARCH" = "arm64" ]; then
    export RUST_TARGET=aarch64-unknown-linux-gnu
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
    export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
elif [ "$TARGETARCH" = "amd64" ]; then
    export RUST_TARGET=x86_64-unknown-linux-gnu
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
    export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
    export CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++
fi
cargo build --release --bin quorra --target $RUST_TARGET
mkdir -p /app/target/release || true
cp /app/target/$RUST_TARGET/release/quorra /app/target/release/quorra
EOT

FROM debian:bullseye-slim AS runtime

RUN apt-get update && apt-get install tini -y
WORKDIR /app
COPY --from=release /app/target/release/quorra /usr/local/bin
RUN /usr/local/bin/quorra --help
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/quorra", "server"]