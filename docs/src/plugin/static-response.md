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
source = { type = "static", config_path = "./static-rest.yaml" }
```

This will configure `/dev/null` to use the static plugin and read the static config from `./static-rest.yaml`. The path in `config_path` is relative to the application path. For production deployments using an absolute path is best.

Next create `./static-rest.yaml` with the contents

```yaml
http:
  - matches:
    - path: /echo
    responses:
      - headers: {}
        body:
          type: raw
          data: '{{ request_body }}'
        status: 200
        weight: 5
      - body:
          type: raw
          data: '{}'
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
          data: |
            {
              "uuid": "{{ uuid }}",
              "id": {{ id }}
            }
        status: 200
```

In this example two endpoints will be registered.

The first `/echo`, which will respond with the request body 5-1 times, the 1 time will be a 500.
The second is `/graphql`, which matches the the operationName is `coolQuery`, and will respond with a JSON object containing `uuid` and `id`.

### Pro Tip

In production we recommend setting ID's to something useful. That way when you are unsure about where a response came from you can trace it by the response headers.

## Magic Values

We use Handlebars to add dynamic values to the response. This is most often used with ID responses.

| Key                   | Description                                                                    |
|-----------------------|--------------------------------------------------------------------------------|
| `uuid`                | A random UUID Version 4                                                        |
| `id`                  | A sequential unsigned number. This value will reset when `/dev/null` restarts. |
| `dev_null_plugin_id`  | The Plugin Id that matched                                                     |
| `dev_null_payload_id` | The Payload Id that matched                                                    |
| `request_body`        | The entire request body                                                        |

## File structure

At the moment, `/dev/null` only supports http requests, so all entries are under `http`.

| Key    | Description                                                                                |
|--------|--------------------------------------------------------------------------------------------|
| `id`   | When not present, will be genreated. The `id` is included as `x-dev-null-plugin-id` header |
| `http` | An arry of Http Payload Config                                                             |

### `http` - HTTP Payload config

| Key         | Description                                                                                 |
|-------------|---------------------------------------------------------------------------------------------|
| `id`        | When not present, will be genreated. The `id` is included as `x-dev-null-payload-id` header |
| `matches`   | An array of matches. See below for configuration options.                                   |
| `responses` | An array of responses. See below for configuration options.                                 |

### `matches` - Request Matches

The `matches` field is used to determine if the response should be sent. _ALL_ options must match or it will be skipped. All matchers are optional, if none are provided, the request will always match.

A description of the matchers.

| Key                      | Description                                                                                                               |
|--------------------------|---------------------------------------------------------------------------------------------------------------------------|
| `path`                   | A [regex][regex] to match against the path. The Regex will be parsed as `^{path}$` to ensure that the path fully matches. |
| `headers`                | A key-value map. The key is the header name, and the value is a [regex][regex] that can be used to match against.         |
| `methods`                | A list of http [methods][methods].                                                                                        |
| `graphql.operation-name` | A [regex][regex] of the graphql operation                                                                                 |

### `responses` - Response Options

A list of possible responses. At least one with a weight of > 0 required.

| Key          | Description                                                                                                                    |
|--------------|--------------------------------------------------------------------------------------------------------------------------------|
| `id`         | When not present, will be genreated. The `id` is included as `x-dev-null-payload-id` header                                    |
| `headers`    | Optional, when set a key-value list of headers that will be included in the response                                           |
| `body.type`  | Either `json` or `raw` depending on the data being responded with                                                              |
| `body.data`  | Is a string that will be used as the response                                                                                  |
| `status`     | The HTTP status response code                                                                                                  |
| `weight`     | Defaults to 1. Used to provide a response ratio compared to other requests. Useful when returning an error with 1% of requests |
| `delay`      | Default to 0. The number of millisecond to wait before responding to the reqeust.                                              |

  [regex]: https://docs.rs/regex/latest/regex/
  [methods]: https://docs.rs/http/latest/http/method/struct.Method.html