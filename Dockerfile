# syntax=docker/dockerfile:1.4
FROM rust:bullseye as builder
RUN <<EOT
#!/usr/bin/env bash
set -euxo pipefail

apt-get update
apt-get install protobuf-compiler -y
EOT

WORKDIR /app
COPY rust-toolchain.toml /app/rust-toolchain.toml
RUN cargo

COPY . /app/
RUN ls vendor
COPY <<EOF /app/.cargo/config.toml 
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

FROM builder as dep_check
RUN rustup toolchain install nightly --allow-downgrade --profile minimal

RUN <<EOT
#!/usr/bin/env bash
set -euxo pipefail

cargo +nightly build --release
cargo +nightly install cargo-udeps --locked
cargo +nightly udeps --release
EOT

FROM builder as test
RUN <<EOT
#!/usr/bin/env bash
set -euxo pipefail

rustup component add rustfmt clippy

cargo test --release
cargo fmt --check
cargo clippy --release
EOT

FROM scratch as check
COPY --from=dep_check /app/Cargo.lock Cargo-dep-check.lock
COPY --from=test /app/Cargo.lock Cargo-test.lock

FROM builder as release
RUN cargo build --release --bin quorra
RUN /app/target/release/quorra --help

FROM debian:bullseye-slim AS runtime

RUN apt-get update && apt-get install tini -y
WORKDIR /app
COPY --from=release /app/target/release/quorra /usr/local/bin
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/quorra", "server"]