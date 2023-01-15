# Configuration

In order to use `/dev/null` you will need to provide a config file.

The simpled config file is

```toml
[http]
address = "127.0.0.1:8080"
```

This will let `/dev/null` to listen on `127.0.0.1`, and on port `8080`.

## Pro Tip

When used in a large application, defining the static plugin multiple times will allow teams to own specific files. Seperate the ownership based on your ownership model.

## Plugins

To configure plugins, you will need to add something like

```toml
[[http.plugin]]
source = { type = "static" }
config = "./static-rest.yaml"
```

For proper values, review the [plugins](./plugins.md) to see how to configure each plugin.
