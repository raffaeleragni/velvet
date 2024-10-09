# Velvet
(original repo: https://github.com/raffaeleragni/velvet)

![crates.io](https://img.shields.io/crates/v/velvet_web)

A layer of republish and small functions to remove some boilerplate on web stacks.

For a reference/example of a project using it: https://github.com/raffaeleragni/veltes

Other sample projects that use velvet:
- https://github.com/raffaeleragni/forumfactor
- https://github.com/raffaeleragni/norush

## Stack used

  - WEB: `Axum`
  - DB: `sqlx`(postgres,sqlite,mysql)
  - Templating: `Askama` (folder templates/)
  - Telemetry: `sentry` supported
  - Metrics: `prometheus` under `/metrics/prometheus`

The askama templates and the static RustEmbed will be compiled in and not required at runtime.

The sqlx migrations are not embedded, and will be needed at runtime.

Proc macros cannot be transferred transitively, so crates need to be added again at project root in order to access them. For example `tokio` or `serde`.

## Base route setup

```rust
use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    App::new().route("/", get(index)).start().await.unwrap();
}

async fn index() -> impl IntoResponse {
    "Hello World"
}
```

## Logging

Default log level is `error`. To change the level use the env var `RUST_LOG=info|debug|warn`.

To get structured logging (`json` logs) pass env var `STRUCTURED_LOGGING=true`.

[example](examples/02_logging.rs)

## Add custom metrics

Metrics available at `/metrics/prometheus`.
The custom metrics will be visible as soon as the first use happens, but only when used after App startup, not before.
For example, all the routes will work when used like this.

[example](examples/03_metrics.rs)

## Add a database

Adding a `.env` file with `DATABASE_URL=sqlite::memory:`, and enabling the feature `sqlite` in crate `velvet_web`.

[example](examples/04_database.rs)

## Use an HTTP Client

[example](examples/05_client.rs)

## Check JWT token (from bearer or cookies)

Adding a `.env` file with `JWT_SECRET=secret` and enabling the feature `auth` in `velvet_web`.

JWK urls are also supported with a different enum initialization `JWT::JWK.setup().await?`.

[example](examples/06_token.rs)

## Support for static files

Need to include crate `rust_embed` as this uses proc macros.

[example](examples/07_statics.rs)

## A more complete example

Using also Askama templates, and JWT through cookie setting.

[example](examples/08_full.rs)

## Testing routes

[example](examples/09_testing.rs)

## Embedded login(and registration) flow

[example](examples/10_login.rs)

## Default routes already implemented

  - Status (no-op): http GET /status/liveness
  - Metrics: http GET /metrics/prometheus

## ENV vars

  - SERVER_BIND: [default] default (0.0.0.0) bind network for which to listen on
  - SERVER_PORT: [number] (default 8080) port for which to listen on
  - DATABASE_URL: postgres://user:pass@host:port/database (if database used) or sqlite::memory:...
  - DATABASE_MAX_CONNECTIONS: [number] (default 1)
  - STRUCTURED_LOGGING: true|false (default false)
  - SENTRY_URL: url inclusive of key for sending telemetry to sentry

## To setup TLS use env vars:

  - TLS=true (or any string)
  - TLS_PEM_CERT=cert.pem
  - TLS_PEM_KEY=key.pem
