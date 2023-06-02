#!/usr/bin/env bash

set -eux

VERSION="${1:-'0.1.1-SNAPSHOT'}"

mkdir -p target/artifacts
cargo install cross

echo "Building x86-64 Linux Artifact"
cross build --release --verbose --target x86_64-unknown-linux-gnu --config package.version=\"${VERSION}\"
ls target/x86_64-unknown-linux-gnu/release/quorra
file target/x86_64-unknown-linux-gnu/release/quorra

echo "Building aarch64 Linux Artifact"
cross build --release --verbose --target aarch64-unknown-linux-gnu --config package.version=\"${VERSION}\"
ls target/aarch64-unknown-linux-gnu/release/quorra
file target/aarch64-unknown-linux-gnu/release/quorra
