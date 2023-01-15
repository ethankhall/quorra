# Static Response

The `static` plugin allows developers to provide static responses for known requests. This is the simplest pluign, and will be used most often.

The static plugin provides a few useful features for developers:
- Multiple responses for a single path, useful for error responses.
- Matching against GraphQL operation names
- JSON formatted responses

## Setup

To configure a static response, two different configurations need to be provided. First telling `/dev/null` about the static plugin.

Within your application config file, usualy `config.toml` add the following snippit.

```toml
[[http.plugin]]
source = { type = "static" }
config = "./static-rest.yaml"
```

This will configure `/dev/null` to use the static plugin and read the static config from `./static-rest.yaml`. The path in `config` is relative to the application path. For production deployments using an absolute path is best.

Next create `./static-rest.yaml` with the contents

```yaml
http:
  - matches:
    - path: /api/v1/foo
      headers:
        host: localhost
      methods:
       - GET
    responses:
      - headers: {}
        body:
          type: json
          json: {}
        status: 200
        weight: 5
      - body:
          type: raw
          bytes: '{}'
        status: 500
  - matches:
    - path: /graphql
      methods:
       - POST
      graphql:  
        operation-name: coolQuery
    responses:
      - headers: {}
        body:
          type: json
          json: {}
        status: 200
```

### Pro Tip

When used in a large application, defining the static plugin multiple times will allow teams to own specific files. Seperate the ownership based on your ownership model.

## The `config` file structure

At the moment, `/dev/null` only supports http requests, so all entries are under `http`.

Within `http` key there are two elements that need to be filled out `matches` and `responses`.