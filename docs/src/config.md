# Configuration

In order to use Quorra you will need to provide a config file.

The simpled config file is

```toml
[http]
address = "127.0.0.1:8080"
```

This will let Quorra to listen on `127.0.0.1`, and on port `8080`.

## Pro Tip

When used in a large application, defining the static plugin multiple times will allow teams to own specific files. Seperate the ownership based on your ownership model.

## Plugins

To configure plugins, you will need to add something like

```toml
[responses]
paths = [
    "foo/**/*.yaml"
]
```

Quorra will then scan the listed files for plugin definitions.

For proper values, review the [plugins](./plugins.md) to see how to configure each plugin.
