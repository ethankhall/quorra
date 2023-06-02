#!/usr/bin/env bash

set -eux

VERSION="${1:-'0.1.1-SNAPSHOT'}"

mkdir -p target/artifacts
cargo build --release --verbose --target aarch64-apple-darwin --config package.version=\"${VERSION}\"
cargo build --release --verbose --target x86_64-apple-darwin --config package.version=\"${VERSION}\"

lipo target/aarch64-apple-darwin/release/quorra target/x86_64-apple-darwin/release/quorra -create -output target/artifacts/quorra

echo "Built a multi-arch binary attarget/artifacts/quorra"
file target/artifacts/quorra
target/artifacts/quorra --help
