# Quorra Service

Faking responses like it's 2001.

See our [docs](./docs/src/SUMMARY.md) for more details. To learn how to develop Quorra checkout our [Developer Docs](./docs/src/developer/index.md)

## Quick Start

Running the webserver: `quorra server --config mocks/config.toml --listen 0.0.0.0:3001`

Convert `har` file into quorra config `qurra convert-har --har ./foo.har --destination ./mocks/foo-service`
