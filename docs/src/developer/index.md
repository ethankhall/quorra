# Developing

In order to work on Quorra you'll need to have Rust installed. We recommend using [`rustup`](https://rustup.rs).

The repo has a default config that can be used to test with. To use this configuration run `cargo run -- -c example/config.toml -d`.

When running, metrics will be send to a local OTEL server. You'll also need to run these services by running `docker compose -f docker-compose.dev.yaml up`.

## Contributing

Be sure changes pass `cargo fmt` and `cargo clippy`.
